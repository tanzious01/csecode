pub fn auth() -> String {
    let my_username = "ptttbaxkieubr69";
    let password = "3jqewclth6gwxk3";
    let proxy = "rp.proxyscrape.com:6060";
    let auth_string = format!("{}:{}@{}", my_username, password, proxy);
    auth_string
}
