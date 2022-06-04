// Непосредственно само включение трассировки макросов после включения фичи в main.rs
// trace_macros!(true);

#[macro_use]
mod macros; // Специально самый первый
mod subtypes;
#[cfg(test)]
mod tests;
mod traits;

pub use self::subtypes::*;
use self::traits::EnvParams;
use any_field_is_some_macro::any_field_is_some;

// TODO: Clap поддерживает и переменные окружения как оказалось

macro_rules! describe_env_values {
    ( $( $val: ident: $type_id:ident ),* ) => {
        #[derive(Debug)]
        #[any_field_is_some]
        pub struct AppEnvValues{
            $( pub $val: std::option::Option<$type_id> ),*
        }
        impl AppEnvValues {
            pub fn parse() -> AppEnvValues {
                let params = AppEnvValues{
                    $( $val: $type_id::try_parse() ),*
                };

                params
            }
            pub fn get_possible_env_variables() -> Vec<&'static str>{
                // TODO: Убрать for, сделать на операторах
                let mut vec = Vec::new();
                $(
                    for key in $type_id::get_available_keys(){
                        vec.push(*key);
                    }
                )*
                vec
            }
        }
    };
}

describe_env_values!(
    git: GitEnvironment,
    amazon: AmazonEnvironment,
    app_center: AppCenterEnvironment,
    google_play: GooglePlayEnvironment,
    google_drive: GoogleDriveEnvironment,
    ios: IOSEnvironment,
    windows: WindowsStoreEnvironment,
    facebook: FacebookInstantEnvironment,
    ssh: SSHEnvironment,
    target_slack: TargetSlackEnvironment,
    result_slack: ResultSlackEnvironment
);
