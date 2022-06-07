use super::upload_result::{UploadResult, UploadResultData};
use crate::{app_parameters::SSHParams, env_parameters::SSHEnvironment};
use image::EncodableLayout;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    str::from_utf8,
};
use tokio::task::{spawn_blocking, JoinHandle};
use tracing::{debug, error};
use which::which;

////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
enum SshError {
    #[error("SSH executable not found on environment with error `{0}`")]
    SshAppNotFound(#[from] which::Error),

    #[error("SSH application spawn failed with error `{err}`")]
    SpawnFail {
        #[source]
        err: std::io::Error,
    },

    #[error("SSH application spawn result wait failed with error `{err}`")]
    SpawnWaitFail {
        #[source]
        err: std::io::Error,
    },

    #[error("SSH command execute failed, context `{context}`: exit code `{exit_code:?}`, std error output `{std_err:?}`")]
    CommandExecutionError {
        context: String,
        exit_code: Option<i32>,
        std_err: Option<String>,
    },

    #[error("Parsing failed, context `{context}`, err `{err}`")]
    UTF8ParsingFailed {
        context: String,
        #[source]
        err: std::str::Utf8Error,
    },

    #[error("Invalid target folder `{0}`")]
    InvalidTargetFolder(&'static str),

    #[error("Source file is missing or invalid path")]
    InvalidSourceFilePath(PathBuf),
}

////////////////////////////////////////////////////////////////////////

/// Контекст для работы SSH выгрузки
struct SshExecuteInfo {
    ssh_executable_path: PathBuf,
    scp_executable_path: PathBuf,
    env_params: SSHEnvironment,
}

/// Выполняем ssh команду, ssh с установкой соединения выполняется каждый раз
fn execute_ssh_command(ssh_info: &SshExecuteInfo, command: &str) -> Result<Output, SshError> {
    #[rustfmt::skip]
    let output = Command::new(&ssh_info.ssh_executable_path)
        .args([
            "-i", &ssh_info.env_params.key_file,
            &format!("{}@{}", ssh_info.env_params.user, ssh_info.env_params.server),
            command
        ])
        .stdin(Stdio::piped())  // Возможны проблемы на винде, поэтому всегда piped
        .stdout(Stdio::piped()) // Возможны проблемы на винде, поэтому всегда piped
        .stderr(Stdio::piped()) // Возможны проблемы на винде, поэтому всегда piped
        .spawn()
        .map_err(|err| SshError::SpawnFail{ err })?
        .wait_with_output()
        .map_err(|err| SshError::SpawnWaitFail{ err })?;

    // Нормально ли все отработало?
    if !output.status.success() {
        // Парсим вывод stderr как текст UTF-8
        let stderr_output = from_utf8(output.stderr.as_bytes())
            .map(|v| v.to_owned())
            .ok();

        // Ошибка
        return Err(SshError::CommandExecutionError {
            context: format!("Command `{}`", command),
            exit_code: output.status.code(),
            std_err: stderr_output,
        });
    }

    Ok(output)
}

/// Выполняем ssh команду, ssh с установкой соединения выполняется каждый раз
fn execute_ssh_command_string_output(
    ssh_info: &SshExecuteInfo,
    command: &str,
) -> Result<String, SshError> {
    let output = execute_ssh_command(ssh_info, command)?;

    // Парсим вывод stdout как текст UTF-8
    from_utf8(output.stdout.as_bytes())
        .map(|v| v.trim_end().to_owned())
        .map_err(|err| SshError::UTF8ParsingFailed {
            context: format!("StdOutput for command `{}`", command),
            err,
        })
}

/// Выполняем ssh команду, ssh с установкой соединения выполняется каждый раз
fn execute_ssh_command_no_output(ssh_info: &SshExecuteInfo, command: &str) -> Result<(), SshError> {
    let _ = execute_ssh_command(ssh_info, command)?;
    Ok(())
}

/// Получаем абсолютный пути к директории на сервере если был передан
/// путь относительно домашней директории
fn get_remote_abs_path<'a>(
    ssh_info: &SshExecuteInfo,
    target_dir: &'a str,
) -> Result<Cow<'a, Path>, SshError> {
    // Создаем Path из переданной строки
    let input_path = Path::new(target_dir);

    // Может быть у нас уже корневая директория и так?
    let result_absolute_folder_path = if input_path.has_root() {
        Cow::Borrowed(input_path)
    }
    // Если директория у нас начинается с `~/`, тогда пытаемся найти полный путь
    else if input_path.starts_with("~/") {
        // Домашняя директория
        let home_path = {
            let home_path_str = execute_ssh_command_string_output(ssh_info, "echo ~")?;
            debug!("Home path: {home_path_str}");
            PathBuf::from(home_path_str)
        };

        Cow::Owned(home_path.join(input_path.strip_prefix("~/").unwrap())) // Unwrap, так как мы уже проверили выше
    } else {
        return Err(SshError::InvalidTargetFolder(
            "Only absolute and '~/' paths are supported",
        ));
    };

    Ok(result_absolute_folder_path)
}

/// Создаем конкретную директорию для файлика на сервере
fn create_result_folder(ssh_info: &SshExecuteInfo, path: &Path) -> Result<(), SshError> {
    let command = format!("mkdir -p {}", path.display());
    debug!("Execute command: {}", command);
    execute_ssh_command_no_output(ssh_info, &command)
}

/// Выполняем копирование файлика на сервер с помощью утилиты scp
fn execute_scp_uploading(
    ssh_info: &SshExecuteInfo,
    target_dir: &Path,
    source_file: &Path,
) -> Result<(), SshError> {
    #[rustfmt::skip]
    let command = Command::new(&ssh_info.scp_executable_path)
        .args([
            "-i", &ssh_info.env_params.key_file,
            &format!("{}", source_file.display()), // TODO: Лишнее выделение памяти
            &format!("{}@{}:{}/", ssh_info.env_params.user, ssh_info.env_params.server, target_dir.display()),
        ])
        .stdin(Stdio::piped())  // Возможны проблемы на винде, поэтому всегда piped
        .stdout(Stdio::piped()) // Возможны проблемы на винде, поэтому всегда piped
        .stderr(Stdio::piped()); // Возможны проблемы на винде, поэтому всегда piped
        
    debug!("Execute upload command: {:?}", command);

    #[rustfmt::skip]
    let output = command.spawn()
        .map_err(|err| SshError::SpawnFail{ err })?
        .wait_with_output()
        .map_err(|err| SshError::SpawnWaitFail { err })?;

    // Нормально ли все отработало?
    if output.status.success() {
        Ok(())
    } else {
        // Парсим вывод stderr как текст UTF-8
        let stderr_output = from_utf8(output.stderr.as_bytes())
            .map(|v| v.to_owned())
            .ok();

        // Ошибка
        Err(SshError::CommandExecutionError {
            context: format!("Source file path `{}`", source_file.display()),
            exit_code: output.status.code(),
            std_err: stderr_output,
        })
    }
}

pub async fn upload_by_ssh(env_params: SSHEnvironment, app_params: SSHParams) -> UploadResult {
    let join: JoinHandle<Result<UploadResultData, SshError>> = spawn_blocking(move || {
        let _span = tracing::info_span!("upload_by_ssh");

        // Сначала находим путь к исполняемому файлику из окружения
        let ssh_executable_path = which("ssh")?;
        debug!("SSH executable found at path: {ssh_executable_path:?}");
        let scp_executable_path = which("scp")?;
        debug!("SCP executable found at path: {scp_executable_path:?}");

        // Данные для работы SSH
        let ssh_info = SshExecuteInfo {
            ssh_executable_path,
            scp_executable_path,
            env_params,
        };

        // Получаем полный путь к целевой директории если надо
        let remote_target_dir = get_remote_abs_path(&ssh_info, &app_params.target_dir)?;
        debug!("Remote target path: {remote_target_dir:?}");

        // Создаем конечную директорию для выгрузки
        create_result_folder(&ssh_info, &remote_target_dir)?;
        debug!("Target directory create success");

        // Пути выгрузки
        let upload_paths: Vec<PathBuf> = app_params.files.into_iter().map(PathBuf::from).collect();

        // Проверяем сразу, что есть все файлы перед загрузкой
        {
            let invalid_path = upload_paths.iter().find(|p| !p.exists() || !p.is_file());
            if let Some(invalid_path) = invalid_path {
                return Err(SshError::InvalidSourceFilePath(invalid_path.to_owned()));
            }
        }

        // Идем по списку файликов, которые надо перекинуть
        let mut filenames = Vec::new();
        filenames.reserve(upload_paths.len());
        for source_path in upload_paths {
            // Делаем выгрузку
            execute_scp_uploading(&ssh_info, &remote_target_dir, &source_path)?;
            debug!("File `{:?}` uploaded", source_path);

            // Добавляем имя файлика к списку выгрузки
            let filename = source_path
                .file_name()
                .and_then(|v| v.to_str())
                .map(|v| v.to_owned())
                .ok_or(SshError::InvalidSourceFilePath(source_path))?;
            filenames.push(filename);
        }

        // Финальное сообщение
        let names_str = filenames.into_iter().fold(String::new(), |mut prev, n| {
            prev.push_str(&format!("\n- {}", n));
            prev
        });
        let message = format!("SSH uploading finished:{}", names_str);

        Ok(UploadResultData {
            install_url: None,
            message: Some(message),
            target: "SSH",
        })
    });

    let result = join.await.expect("SSH join failed");
    match result {
        Ok(res) => Ok(res),
        Err(err) => Err(Box::new(err)),
    }
}
