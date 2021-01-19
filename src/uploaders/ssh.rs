use std::{
    string::{
        FromUtf8Error
    },
    path::{
        Path
    },
    io::{
        self,
    },
    net::{
        TcpStream,
        SocketAddrV4,
        Ipv4Addr
    },
    fmt::{
        Display,
        Formatter,
        self
    },
    error::{
        self
    }
};
use tokio::{
    task::{
        spawn_blocking,
        JoinError,
        JoinHandle
    }
};
use log::{
    debug,
    error,
    trace
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
    DNSError(ResolveError),
    EmptyDNSAddresses,
    PrivateKeyNotFound,
    AuthFailed,
    InvalidServerAddr
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
        SshError::DNSError(err)
    }
}

////////////////////////////////////////////////////////////////////////

fn try_to_auth(user: String, 
              pass: Option<String>, 
              key_file: Option<String>, 
              session: &Session) -> Result<(), SshError>{
    let mut authentificated = false;
    if !authentificated {
        if let Some(ref password) = pass {
            debug!("SSH password for auth");
            authentificated = session.userauth_password(&user, &password).is_ok();
        }
    }
    if !authentificated {
        if let Some(ref key_path) = key_file {
            let path = Path::new(&key_path);
            if !path.exists(){
                error!("SSH private key for does not exist");
                return Err(SshError::PrivateKeyNotFound);
            }
            debug!("SSH private key for auth");
            authentificated = session.userauth_pubkey_file(&user, 
                                                           None,
                                                           path,
                                                           None).is_ok();
        }
    }
    if !authentificated {
        debug!("NO auth info");
        return Err(SshError::AuthFailed);
    }
    Ok(())
}

pub async fn upload_by_ssh(env_params: SSHEnvironment, 
                           app_params: SSHParams) -> UploadResult {
    let join: JoinHandle<Result<UploadResultData, SshError>> = spawn_blocking(move || {
        // Тип аддреса
        let addr = if let Ok(_) = &env_params.server.parse::<SocketAddrV4>(){
            env_params.server
        }else if let Ok(addr) = &env_params.server.parse::<Ipv4Addr>(){
            SocketAddrV4::new(addr.clone(), 22).to_string()
        }else {
            let resolver = Resolver::new(ResolverConfig::default(), 
                                         ResolverOpts::default())?;
            let response = resolver
                .lookup_ip(env_params.server.clone())?;
            let address = response
                .iter()
                .next()
                .ok_or(SshError::EmptyDNSAddresses)?;
            address.to_string()
        };

        debug!("SSH server address: {}", addr);

        let stream = TcpStream::connect(addr)?;

        debug!("Stream created");

        let mut session = Session::new()?;
        session.set_tcp_stream(stream);
        
        session.handshake()?;
        debug!("Handshade complete");

        try_to_auth(env_params.user, env_params.pass, env_params.key_file, &session)?;
        debug!("Auth complete");


        Ok(UploadResultData{
            install_url: None,
            message: None,
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
        pretty_env_logger::formatted_builder()
            // .is_test(true)
            .filter_level(log::LevelFilter::Trace)
            .try_init()
            .ok();

        let env_params = SSHEnvironment{
            server: "192.168.1.2".to_owned(), // TODO: DNS name
            user: "pi".to_owned(),
            key_file: Some("/Users/devnul/.ssh/id_rsa".to_owned()),
            pass: None,
        };
        let app_params = SSHParams{
            files: vec![
                "qeqew".to_owned()
            ],
            target_dir: "test".to_owned()
        };

        upload_by_ssh(env_params, app_params)
            .await
            .expect("SSH uploading failed");
    }
}