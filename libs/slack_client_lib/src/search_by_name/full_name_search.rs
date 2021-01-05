// use log::{
//     error
// };
use super::{
    user_info::{
        UserInfo
    }
};

pub fn search_by_fullname(full_users_list: Vec<UserInfo>, user_lowercase: &str) -> Option<UserInfo>{
    // Структура юзера с приоритетом
    #[derive(Debug)]
    struct UserInfoWithPriority{
        priority: i32,
        info: UserInfo,
    }

    let mut found_users: Vec<UserInfoWithPriority> = Vec::new(); 

    let search_parts: Vec<&str> = user_lowercase.split(' ').collect();
    for user_info in full_users_list { // Объекты будут перемещать владение в user_info        
        // Проверяем полное имя
        if let Some(ref real_name_src) = user_info.real_name {
            let real_name = real_name_src.to_lowercase();

            // Нашли сразу же
            if real_name == user_lowercase {
                return Some(user_info);
            }else{
                let mut possible_val = UserInfoWithPriority{
                    priority: 0,
                    info: user_info,
                };
                let test_name_components = real_name.split(' '); // split создает итератор с &str
                for test_part in test_name_components { // Здесь у нас владение перемещается полностью в итерируемые элементы, test_name_components пустой после
                    for search_part in search_parts.iter() { // Тут мы итерируемся по элементам
                        if test_part == *search_part {
                            possible_val.priority += 1;
                        }
                    }
                }
                
                if possible_val.priority > 0 {
                    found_users.push(possible_val);
                }
            }
        }
    }

    // TODO: работает неправильно!
    /*found_users.sort_by_key(|user: &UserInfoWithPriority|{
        -user.priority
    });*/

    found_users.sort_by(|val1, val2|-> std::cmp::Ordering {
        if val1.priority < val2.priority{
            return std::cmp::Ordering::Greater;
        } else if val1.priority > val2.priority{
            return std::cmp::Ordering::Less;
        }
        std::cmp::Ordering::Equal
    });

    // Вернем просто первый элемент
    return found_users
        .into_iter()
        .take(1)
        .next()
        .map(|user_info| user_info.info);
    /*for user_info in found_users {
        return Some(user_info.info);
    }*/
} 

// этот модуль включается только при тестировании mod tests {
#[cfg(test)]
pub mod tests{
    use crate::{
        search_by_name::{
            UserInfo,
        },
        tests_helpers::{
            generate_test_users
        }
    };
    use super::{
        *
    };

    #[test]
    fn test_full_name_serach(){
        let test_names_map = generate_test_users();
        let test_names_vec: Vec<UserInfo> = test_names_map
            .iter()
            .map(|(_, val)|{
                val.clone()
            })
            .collect();

        assert_eq!(search_by_fullname(test_names_vec.clone(), "pavel ershov").map(|val| val.id), 
                   Some(test_names_map["pershov"].id.clone()));
                
        assert_eq!(search_by_fullname(test_names_vec.clone(), "ershov pavel").map(|val| val.id), 
                   Some(test_names_map["pershov"].id.clone()));

        assert_eq!(search_by_fullname(test_names_vec.clone(), "user unknown").map(|val| val.id), 
                   None);

        assert_eq!(search_by_fullname(test_names_vec.clone(), "unknown").map(|val| val.id), 
                   None);
    }
}