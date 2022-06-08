use crate::{
    error::MicrosoftAzureError,
    request_builder::RequestBuilder,
    responses::{DataOrErrorResponse, FlightSubmissionCommitResponse, SubmissionStatusResponse},
};
use log::debug;

/// Данный метод занимается тем, что коммитит изменения на сервере
/// Описание: `https://docs.microsoft.com/en-us/windows/uwp/monetize/commit-a-flight-submission`
pub async fn commit_changes(request_builder: &RequestBuilder) -> Result<(), MicrosoftAzureError> {
    debug!("Microsoft Azure: commit request");

    let new_info = request_builder
        .clone()
        .method(reqwest::Method::POST)
        .submission_command("commit".to_string())
        .build()
        .await?
        .header(reqwest::header::CONTENT_LENGTH, "0")
        .send()
        .await?
        // .error_for_status()?
        .json::<DataOrErrorResponse<FlightSubmissionCommitResponse>>()
        .await?
        .into_result()?;

    debug!("Microsoft Azure: commit response {:#?}", new_info);

    if !new_info.status.eq("CommitStarted") {
        return Err(MicrosoftAzureError::InvalidCommitStatus(new_info.status));
    }

    Ok(())
}

/// C помощью данного метода мы ждем завершения выполнения коммита
/// Описание: `https://docs.microsoft.com/en-us/windows/uwp/monetize/get-status-for-a-flight-submission`
pub async fn wait_commit_finished(request_builder: &RequestBuilder) -> Result<(), MicrosoftAzureError> {
    debug!("Microsoft Azure: wait submission commit result");

    loop {
        let status_response = request_builder
            .clone()
            .method(reqwest::Method::GET)
            .submission_command("status".to_string())
            .build()
            .await?
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?
            // .error_for_status()?
            .json::<DataOrErrorResponse<SubmissionStatusResponse>>()
            .await?
            .into_result()?;

        debug!(
            "Microsoft Azure: submission status response {:#?}",
            status_response
        );

        match status_response.status.as_str() {
            // Нормальное состояние для ожидания
            "CommitStarted" => {
                tokio::time::sleep(std::time::Duration::from_secs(15)).await;
            }

            // Коммит прошел успешно, прерываем ожидание
            "PreProcessing" | "PendingPublication" | "Certification" | "Publishing"
            | "Published" | "Release" => {
                break;
            }

            // Ошибочный статус - ошибка
            "CommitFailed"
            | "None"
            | "Canceled"
            | "PublishFailed"
            | "PreProcessingFailed"
            | "CertificationFailed"
            | "ReleaseFailed" => {
                return Err(MicrosoftAzureError::CommitFailed(status_response));
            }

            // Неизвестный статус - ошибка
            _ => {
                return Err(MicrosoftAzureError::InvalidCommitStatus(
                    status_response.status,
                ));
            }
        }
    }

    Ok(())
}
