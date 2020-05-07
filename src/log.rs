/*
 * author: Aleksei Kozadaev (2020)
 */

use chrono::prelude::Local;

pub fn get_time() -> String {
    Local::now().format("%H:%M:%S").to_string()
}
