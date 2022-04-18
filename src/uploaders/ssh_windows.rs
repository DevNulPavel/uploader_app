use super::upload_result::{UploadResult, UploadResultData};
use crate::{app_parameters::SSHParams, env_parameters::SSHEnvironment};
use std::{
    error::{self},
    fmt::{self, Display, Formatter},
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddrV4, TcpStream},
    path::{Path, PathBuf},
};
use tokio::task::{spawn_blocking, JoinHandle};
use tracing::{
    debug,
    error,
    // trace,
    instrument,
};

////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
enum SshError {
    #[error("SSH executable not found at environment with error {0}")]
    SshNotFound(#[from] which::Error),
}

////////////////////////////////////////////////////////////////////////

pub async fn upload_by_ssh(env_params: SSHEnvironment, app_params: SSHParams) -> UploadResult {
    let join: JoinHandle<Result<UploadResultData, SshError>> = spawn_blocking(move || {
        let _span = tracing::info_span!("upload_by_ssh");

        // Сначала находим путь к исполняемому файлику из окружения
        let ssh_executable_path = which::which("ssh")?;
        debug!("SSH executable found at path: {}", ssh_executable_path.display());

        unimplemented!()
    });

    let result = join.await.expect("SSH join failed");
    match result {
        Ok(res) => Ok(res),
        Err(err) => Err(Box::new(err)),
    }
}
