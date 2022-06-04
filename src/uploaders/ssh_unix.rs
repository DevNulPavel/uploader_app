use std::{
    path::{
        Path,
        PathBuf
    },
    io::{
        self,
        Read,
        Write as IOWrite
    },
    net::{
        TcpStream,
        SocketAddrV4,
        Ipv4Addr,
        IpAddr
    },
    fmt::{
        Display,
        Formatter,
        self,
        Write as FmtWrite,
        Error as FmtError
    },
    error::{
        self
    }
};
use tokio::{
    task::{
        spawn_blocking,
        JoinHandle
    }
};
use tracing::{
    debug,
    error,
    // trace,
    instrument
};
use ssh2::{
    Session
};
use trust_dns_resolver::{
    config::{
        ResolverConfig,
        ResolverOpts
    },
    error::{
        ResolveError,
    },
    Resolver
};
use crate::{
    app_parameters::{
        SSHParams
    },
    env_parameters::{
        SSHEnvironment
    }
};
use super::{
    upload_result::{
        UploadResult,
        UploadResultData
    }
};

////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum SshError{
    SshErr(ssh2::Error),
    IO(io::Error),
    DNSError(Box<ResolveError>),
    IpV6IsUnsupported,
    EmptyDNSAddresses,
    PrivateKeyNotFound,
    AuthFailed,
    DirectoryCreateFailed(i32),
    InvalidTargetFolder(&'static str),
    InvalidFilePath(PathBuf),
    FormattingError(FmtError)
}
impl Display for SshError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl error::Error for SshError {
}
impl From<ssh2::Error> for SshError{
    fn from(err: ssh2::Error) -> Self {
        SshError::SshErr(err)
    }
}
impl From<io::Error> for SshError{
    fn from(err: io::Error) -> Self {
        SshError::IO(err)
    }
}
impl From<ResolveError> for SshError{
    fn from(err: ResolveError) -> Self {
        SshError::DNSError(Box::new(err))
    }
}
impl From<FmtError> for SshError{
    fn from(err: FmtError) -> Self {
        SshError::FormattingError(err)
    }
}

////////////////////////////////////////////////////////////////////////

#[instrument]
fn get_valid_address(server: String) -> Result<String, SshError> {
    let addr = if server.parse::<SocketAddrV4>().is_ok(){
        server
    }else if let Ok(addr) = &server.parse::<Ipv4Addr>(){
        SocketAddrV4::new(*addr, 22).to_string()
    }else {
        let resolver = Resolver::new(ResolverConfig::default(), 
                                     ResolverOpts::default())?;
        let response = resolver
            .lookup_ip(server.clone())?;
        let address = response
            .iter()
            .next()
            .ok_or(SshError::EmptyDNSAddresses)?;
        match address {
            IpAddr::V4(v4) => {
                SocketAddrV4::new(v4, 22).to_string()
            },
            _ => {
                return Err(SshError::IpV6IsUnsupported);
            }
        }
    };
    Ok(addr)
}

#[instrument(skip(session))]
fn try_to_auth(user: String, 
               key_file: String, 
               session: &Session) -> Result<(), SshError>{

    let path = Path::new(&key_file);
    if !path.exists(){
        error!("SSH private key for does not exist");
        return Err(SshError::PrivateKeyNotFound);
    }
    debug!("SSH private key for auth");
    let authentificated = session.userauth_pubkey_file(&user, 
                                                       None,
                                                       path,
                                                       None).is_ok();
    if !authentificated {
        debug!("NO auth info");
        return Err(SshError::AuthFailed);
    }
    Ok(())
}

#[instrument(skip(session))]
fn get_remote_abs_path(target_dir: &str, session: &Session) -> Result<PathBuf, SshError> {
    let input_path = Path::new(target_dir);
    let result_absolute_folder_path = if input_path.has_root() {
        input_path.to_owned()
    }else if input_path.starts_with("~/"){
        let mut channel = session.channel_session()?;
        channel.exec("echo ~")?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.close()?;
        let res = PathBuf::new()
            .join(output.trim())
            .join(input_path.strip_prefix("~/").unwrap()); // Unwrap, так как мы уже проверили выше
        res
    }else{
        return Err(SshError::InvalidTargetFolder("Only absolute and '~/' path supported"));
    };
    Ok(result_absolute_folder_path)
}

#[instrument(skip(session))]
fn create_result_folder(path: &Path, session: &Session) -> Result<(), SshError>{
    // TODO: Exit status всегда 0 даже если есть папка
    /*let folder_exist = {
        let mut channel = session.channel_session()?;
        channel.exec(&format!("test -d {}", absolute_path.display()))?;
        let status = channel.exit_status()?;
        channel.close()?;
        debug!("Directory exist status: {}", status);
        status != 0
    };*/
    let folder_exist = false;
    if !folder_exist {
        let mut channel = session.channel_session()?;
        channel.exec(&format!("mkdir -p {}", path.display()))?;
        let status = channel.exit_status()?;
        if status != 0 {
            return Err(SshError::DirectoryCreateFailed(status));
        }
        channel.close()?;
        debug!(dir = %path.display(), "Directory created");
    }
    Ok(())
}

#[instrument(skip(session, result_absolute_folder_path, paths))]
fn upload_files<'a, P>(session: &Session, 
                       result_absolute_folder_path: &Path, 
                       paths: &'a [P]) -> Result<Vec<&'a str>, SshError>
where 
    P: AsRef<Path>
{
    let mut local_filenames = vec![];
    local_filenames.reserve(paths.len());
    for local_path in paths.iter(){
        let local_path = local_path.as_ref();
        let local_file_name = local_path
            .file_name()    
            .ok_or_else(||{
                SshError::InvalidFilePath(local_path.to_path_buf())
            })?
            .to_str()
            .ok_or_else(||{
                SshError::InvalidFilePath(local_path.to_path_buf())
            })?;
        local_filenames.push(local_file_name);
        let remote_path = result_absolute_folder_path
                .join(local_file_name);
        debug!(local = %local_path.display(), 
               remote = %remote_path.display(), 
               "Uploading start");

        let mut file = std::fs::File::open(&local_path)?;
        let size = file.metadata()?.len();
        let mut remote_file = session.scp_send(&remote_path, 0o644, size, None)?;
        
        let mut buffer = [0; 1024*64];
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0{
                break;
            }
            remote_file.write_all(&buffer[0..count])?;
        }

        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;

        debug!(local = %local_path.display(), 
               remote = %remote_path.display(), 
               "Uploading finished");
    }
    Ok(local_filenames)
}

pub async fn upload_by_ssh(env_params: SSHEnvironment, 
                           app_params: SSHParams) -> UploadResult {

    let join: JoinHandle<Result<UploadResultData, SshError>> = spawn_blocking(move || {
        let _span = tracing::info_span!("upload_by_ssh");

        // Тип аддреса
        let addr = get_valid_address(env_params.server)?;
        debug!(%addr, "SSH server address");

        let stream = TcpStream::connect(addr)?;
        debug!("Stream created");

        let mut session = Session::new()?;
        session.set_tcp_stream(stream);
        session.handshake()?;
        debug!("Handshade complete");

        try_to_auth(env_params.user, env_params.key_file, &session)?;
        debug!("Auth complete");

        // Абсолютный путь на сервере к папке
        let result_absolute_folder_path = get_remote_abs_path(&app_params.target_dir, &session)?;
        debug!(?result_absolute_folder_path, "Absolute path");
        
        // Создание папки если надо
        create_result_folder(&result_absolute_folder_path, &session)?;

        // Пути выгрузки
        let paths: Vec<PathBuf> = app_params
            .files
            .into_iter()
            .map(|p|{
                PathBuf::from(&p)
            })
            .collect();
        // Проверяем сразу, что есть все файлы перед загрузкой
        {
            let invalid_path = paths.iter().find(|p|{
                !p.exists()
            });
            if let Some(invalid_path) = invalid_path {
                return Err(SshError::InvalidFilePath(invalid_path.to_owned()));
            }
        }
        // Грузим
        let local_filenames = upload_files(&session, &result_absolute_folder_path, &paths)?;

        // Финальное сообщение
        let names_str = local_filenames
            .into_iter()
            .try_fold(String::new(), |mut prev, n|{
                write!(prev, "\n- {}", n)?;
                Result::<String, std::fmt::Error>::Ok(prev)
            })?;
        let message = format!("SSH uploading finished:{}", names_str);

        Ok(UploadResultData{
            install_url: None,
            message: Some(message),
            target: "SSH"
        })
    });

    let result = join.await.expect("SSH join failed");
    match result {
        Ok(res) => Ok(res),
        Err(err) => Err(Box::new(err))
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[tokio::test]
    async fn test_ssh_uploader(){
        /*pretty_env_logger::formatted_builder()
            // .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init()
            .ok();*/

        let env_params = SSHEnvironment{
            server: "10.51.254.143".to_owned(), // TODO: DNS name
            user: "jenkins".to_owned(),
            key_file: "/Users/devnul/.ssh/id_rsa".to_owned()
        };
        let app_params = SSHParams{
            files: vec![
                "/Users/devnul/Downloads/Discord.dmg".to_owned()
            ],
            target_dir: "/volume1/builds/pi2-gplay/test_folder".to_owned()
        };

        upload_by_ssh(env_params, app_params)
            .await
            .expect("SSH uploading failed");
    }
}