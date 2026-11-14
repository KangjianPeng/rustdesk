use base::config::keys::{self, *};
#[cfg(any(target_os = "android", target_os = "ios"))]
use hbb_common::password_security;
use hbb_common::{
    allow_err,
    bytes::Bytes,
    config::{self, Config, LocalConfig, CONNECT_TIMEOUT, RENDEZVOUS_PORT},
    directories_next,
    futures::future::join_all,
    log,
    rendezvous_proto::*,
    tokio,
};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use hbb_common::{
    sleep,
    tokio::{sync::mpsc, time},
};
use serde_derive::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::common::SOFTWARE_UPDATE_URL;
#[cfg(not(any(target_os = "ios")))]
use crate::ipc;

type Message = RendezvousMessage;

#[derive(Clone, Debug, Serialize)]
pub struct UiStatus {
    pub status_num: i32,
    pub key_confirmed: bool,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub mouse_time: i64,
    pub id: String,
}

lazy_static::lazy_static! {
    static ref UI_STATUS : Arc<Mutex<UiStatus>> = Arc::new(Mutex::new(UiStatus{
        status_num: 0,
        key_confirmed: false,
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        mouse_time: 0,
        id: "".to_owned(),
    }));
    static ref TEMPORARY_PASSWD : Arc<Mutex<String>> = Arc::new(Mutex::new("".to_owned()));
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
lazy_static::lazy_static! {
    static ref OPTIONS : Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(Config::get_options()));
    pub static ref SENDER : Mutex<mpsc::UnboundedSender<ipc::Data>> = Mutex::new(check_connect_status(true));
}

#[cfg(target_os = "windows")]
lazy_static::lazy_static! {
    pub static ref IS_FILE_TRANSFER_ENABLED: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
}

#[inline]
pub fn get_option<T: AsRef<str>>(key: T) -> String {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let map = OPTIONS.lock().unwrap();
        if let Some(v) = map.get(key.as_ref()) {
            v.to_owned()
        } else {
            "".to_owned()
        }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Config::get_option(key.as_ref())
    }
}

#[inline]
pub fn use_texture_render() -> bool {
    false
}

#[inline]
pub fn get_local_option(key: String) -> String {
    crate::get_local_option(&key)
}

#[inline]
pub fn get_builtin_option(key: &str) -> String {
    crate::get_builtin_option(key)
}

#[inline]
pub fn set_local_option(key: String, value: String) {
    LocalConfig::set_option(key.clone(), value);
}

/// Resolve relative avatar path (e.g. "/avatar/xxx") to absolute URL
/// by prepending the API server address.
pub fn resolve_avatar_url(avatar: String) -> String {
    let avatar = avatar.trim().to_owned();
    if avatar.starts_with('/') {
        let api_server = get_api_server();
        if !api_server.is_empty() {
            return format!("{}{}", api_server.trim_end_matches('/'), avatar);
        }
    }
    avatar
}

#[cfg(any(target_os = "android", target_os = "ios", feature = "dioxus-ui"))]
#[inline]
pub fn get_local_ui_option(key: String) -> String {
    LocalConfig::get_flutter_option(&key)
}

#[cfg(any(target_os = "android", target_os = "ios", feature = "dioxus-ui"))]
#[inline]
pub fn set_local_ui_option(key: String, value: String) {
    LocalConfig::set_flutter_option(key, value);
}

pub fn try_set_options(m: HashMap<String, String>) -> hbb_common::ResultType<()> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        if !crate::trusted_devices::config::changes_authentication(&m) {
            *OPTIONS.lock().unwrap() = m.clone();
            hbb_common::allow_err!(ipc::set_options(m));
            return Ok(());
        }
        ipc::set_options(m.clone())?;
        *OPTIONS.lock().unwrap() = m;
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    crate::trusted_devices::config::set_options(m)?;
    Ok(())
}

pub fn set_auth_option(key: String, value: String) -> hbb_common::ResultType<()> {
    if key == keys::OPTION_TRUSTED_DEVICE_EXPIRATION
        && !matches!(value.as_str(), "" | "1" | "7" | "30" | "90" | "0")
    {
        hbb_common::bail!("Invalid trusted-device expiration period");
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let mut options = ipc::get_options();
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let mut options = Config::get_options();
    if value.is_empty() {
        options.remove(&key);
    } else {
        options.insert(key, value);
    }
    try_set_options(options)
}

#[inline]
pub fn set_option(key: String, value: String) {
    if key == "2fa"
        || key == keys::OPTION_ENABLE_TRUSTED_DEVICES
        || key == keys::OPTION_TRUSTED_DEVICE_EXPIRATION
    {
        hbb_common::allow_err!(set_auth_option(key, value));
        return;
    }
    if &key == "stop-service" {
        #[cfg(target_os = "macos")]
        {
            let is_stop = value == "Y";
            if is_stop && crate::platform::uninstall_service(true, false) {
                return;
            }
        }
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        {
            if crate::platform::is_installed() {
                if value == "Y" {
                    if crate::platform::uninstall_service(true, false) {
                        return;
                    }
                } else {
                    if crate::platform::install_service() {
                        return;
                    }
                }
                return;
            }
        }
    } else if &key == "audio-input" {
        #[cfg(not(target_os = "ios"))]
        crate::audio_service::restart();
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let mut options = OPTIONS.lock().unwrap();
        if value.is_empty() {
            options.remove(&key);
        } else {
            options.insert(key.clone(), value.clone());
        }
        ipc::set_options(options.clone()).ok();
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _nat = crate::CheckTestNatType::new();
        Config::set_option(key, value);
    }
}

/// Async settings entry point for UI handlers already running inside Tokio.
pub async fn set_option_async(key: String, value: String) -> hbb_common::ResultType<()> {
    if key == keys::OPTION_TRUSTED_DEVICE_EXPIRATION
        && !matches!(value.as_str(), "" | "1" | "7" | "30" | "90" | "0")
    {
        hbb_common::bail!("Invalid trusted-device expiration period");
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if matches!(
        key.as_str(),
        "2fa" | keys::OPTION_ENABLE_TRUSTED_DEVICES | keys::OPTION_TRUSTED_DEVICE_EXPIRATION
    ) {
        let mut options = ipc::get_options_async().await;
        update_option_value(&mut options, &key, &value);
        ipc::set_options_async(options.clone()).await?;
        *OPTIONS.lock().unwrap() = options;
    } else {
        let options = {
            let mut options = OPTIONS.lock().unwrap();
            update_option_value(&mut options, &key, &value);
            options.clone()
        };
        if crate::trusted_devices::config::changes_authentication(&options) {
            ipc::set_options_async(options.clone()).await?;
            *OPTIONS.lock().unwrap() = options;
        } else {
            if let Err(error) = ipc::set_options_async(options).await {
                log::warn!("Failed to synchronize a setting with the RustDesk service: {error:#}");
            }
        }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _nat = crate::CheckTestNatType::new();
        if matches!(
            key.as_str(),
            "2fa" | keys::OPTION_ENABLE_TRUSTED_DEVICES | keys::OPTION_TRUSTED_DEVICE_EXPIRATION
        ) {
            let mut options = Config::get_options();
            update_option_value(&mut options, &key, &value);
            crate::trusted_devices::config::set_options(options)?;
        } else {
            Config::set_option(key, value);
        }
    }
    Ok(())
}

fn update_option_value(options: &mut HashMap<String, String>, key: &str, value: &str) {
    if value.is_empty() {
        options.remove(key);
    } else {
        options.insert(key.to_owned(), value.to_owned());
    }
}

#[cfg(target_os = "windows")]
#[inline]
pub fn is_installed() -> bool {
    crate::platform::is_installed()
}

#[inline]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn get_connect_status() -> UiStatus {
    UI_STATUS.lock().unwrap().clone()
}

#[inline]
pub fn temporary_password() -> String {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    return password_security::temporary_password();
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    return TEMPORARY_PASSWD.lock().unwrap().clone();
}

#[inline]
pub fn update_temporary_password() {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    password_security::update_temporary_password();
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    allow_err!(ipc::update_temporary_password());
}

#[inline]
pub fn get_fav() -> Vec<String> {
    LocalConfig::get_fav()
}

#[inline]
pub fn store_fav(fav: Vec<String>) {
    LocalConfig::set_fav(fav);
}

#[inline]
pub fn get_new_version() -> String {
    SOFTWARE_UPDATE_URL
        .lock()
        .unwrap()
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_owned()
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[inline]
pub fn get_app_name() -> String {
    crate::get_app_name()
}

#[inline]
pub fn get_lan_peers() -> Vec<HashMap<&'static str, String>> {
    config::LanPeers::load()
        .peers
        .iter()
        .map(|peer| {
            HashMap::<&str, String>::from_iter([
                ("id", peer.id.clone()),
                ("username", peer.username.clone()),
                ("hostname", peer.hostname.clone()),
                ("platform", peer.platform.clone()),
            ])
        })
        .collect()
}

// Preserve relative paths for existing configurations and only remove accidental
// surrounding whitespace. Config values are not shell-expanded (for example, `~`).
fn trim_video_save_directory(value: &str) -> Option<&str> {
    let value = value.trim();
    if !value.is_empty() {
        Some(value)
    } else {
        None
    }
}

// A Windows service typically runs with System32 as its working directory, so
// require an absolute path to avoid resolving recordings there unexpectedly.
#[cfg(any(windows, test))]
fn validate_windows_service_video_save_directory(value: &str) -> Option<&str> {
    let value = trim_video_save_directory(value)?;
    if std::path::Path::new(value).is_absolute() {
        Some(value)
    } else {
        None
    }
}

#[inline]
pub fn video_save_directory(root: bool) -> String {
    let appname = crate::get_app_name();
    // ui process can show it correctly Once vidoe process created it.
    let try_create = |path: &std::path::Path| {
        if !path.exists() {
            std::fs::create_dir_all(path).ok();
        }
        if path.exists() {
            path.to_string_lossy().to_string()
        } else {
            "".to_string()
        }
    };

    if root {
        // Currently, only installed windows run as root
        #[cfg(windows)]
        {
            let dir = Config::get_option(OPTION_WINDOWS_SERVICE_VIDEO_SAVE_DIRECTORY);
            if let Some(dir) = validate_windows_service_video_save_directory(&dir) {
                return dir.to_owned();
            }
            if !dir.trim().is_empty() {
                log::warn!(
                    "Ignoring {OPTION_WINDOWS_SERVICE_VIDEO_SAVE_DIRECTORY}: path must be absolute"
                );
            }
            let drive = std::env::var("SystemDrive").unwrap_or("C:".to_owned());
            let dir =
                std::path::PathBuf::from(format!("{drive}\\ProgramData\\{appname}\\recording",));
            return dir.to_string_lossy().to_string();
        }
    }
    // Get directory from config file otherwise --server will use the old value from global var.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let dir = LocalConfig::get_option_from_file(OPTION_VIDEO_SAVE_DIRECTORY);
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let dir = LocalConfig::get_option(OPTION_VIDEO_SAVE_DIRECTORY);
    if let Some(dir) = trim_video_save_directory(&dir) {
        return dir.to_owned();
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    if let Ok(home) = config::APP_HOME_DIR.read() {
        let mut path = home.to_owned();
        path.push_str(format!("/{appname}/ScreenRecord").as_str());
        let dir = try_create(&std::path::Path::new(&path));
        if !dir.is_empty() {
            return dir;
        }
    }

    if let Some(user) = directories_next::UserDirs::new() {
        if let Some(video_dir) = user.video_dir() {
            let dir = try_create(&video_dir.join(&appname));
            if !dir.is_empty() {
                return dir;
            }
            if video_dir.exists() {
                return video_dir.to_string_lossy().to_string();
            }
        }
        if let Some(desktop_dir) = user.desktop_dir() {
            if desktop_dir.exists() {
                return desktop_dir.to_string_lossy().to_string();
            }
        }
        let home = user.home_dir();
        if home.exists() {
            return home.to_string_lossy().to_string();
        }
    }

    // same order as above
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if let Some(home) = crate::platform::get_active_user_home() {
        let name = if cfg!(target_os = "macos") {
            "Movies"
        } else {
            "Videos"
        };
        let video_dir = home.join(name);
        let dir = try_create(&video_dir.join(&appname));
        if !dir.is_empty() {
            return dir;
        }
        if video_dir.exists() {
            return video_dir.to_string_lossy().to_string();
        }
        let desktop_dir = home.join("Desktop");
        if desktop_dir.exists() {
            return desktop_dir.to_string_lossy().to_string();
        }
        if home.exists() {
            return home.to_string_lossy().to_string();
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let dir = try_create(&parent.join("videos"));
            if !dir.is_empty() {
                return dir;
            }
            // basically exist
            return parent.to_string_lossy().to_string();
        }
    }
    Default::default()
}

#[inline]
pub fn get_api_server() -> String {
    crate::get_api_server(
        get_option("api-server"),
        get_option("custom-rendezvous-server"),
    )
}

pub enum DeployResult {
    Ok,
    NotEnabled,
    InvalidInput,
    IdTaken(String),
    Error(String),
}

impl DeployResult {
    pub fn message(&self) -> String {
        match self {
            Self::Ok => "".to_owned(),
            Self::NotEnabled => "The server does not require explicit deployment.".to_owned(),
            Self::InvalidInput => "Invalid input.".to_owned(),
            Self::IdTaken(id) => {
                format!(
                    "Id `{}` is already used by another machine on the server.",
                    id
                )
            }
            Self::Error(err) => err.clone(),
        }
    }
}

pub fn deploy_device(token: String, new_id: Option<String>) -> DeployResult {
    if Config::no_register_device() {
        return DeployResult::Error("Cannot deploy an unregistrable device!".to_owned());
    }
    let token = token.trim();
    if token.is_empty() {
        return DeployResult::Error("token is required!".to_owned());
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let local_id = Config::get_id();
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let local_id = ipc::get_id();
    let id_to_deploy = new_id.clone().unwrap_or_else(|| local_id.clone());
    let uuid = crate::encode64(hbb_common::get_uuid());
    let pk = crate::encode64(Config::get_key_pair().1);
    let body = serde_json::json!({
        "id": id_to_deploy,
        "uuid": uuid,
        "pk": pk,
    });
    let header = "Authorization: Bearer ".to_owned() + token;
    let url = get_api_server() + "/api/devices/deploy";
    let text = match crate::post_request_sync(url, body.to_string(), &header) {
        Ok(text) => text,
        Err(err) => return DeployResult::Error(format!("Request failed: {}", err)),
    };
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
    match parsed["result"].as_str().unwrap_or("") {
        "OK" => {
            if let Some(new_id) = new_id {
                if new_id != local_id {
                    #[cfg(any(target_os = "android", target_os = "ios"))]
                    {
                        Config::set_key_confirmed(false);
                        Config::set_id(&new_id);
                    }
                    #[cfg(not(any(target_os = "android", target_os = "ios")))]
                    if let Err(err) = ipc::set_config("id", new_id) {
                        return DeployResult::Error(format!(
                            "Failed to persist deployed id locally: {}",
                            err
                        ));
                    }
                }
            }
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if let Err(err) = ipc::notify_deployed() {
                log::warn!("Failed to notify deployed state: {}", err);
            }
            #[cfg(target_os = "android")]
            {
                crate::rendezvous_mediator::NEEDS_DEPLOY
                    .store(false, std::sync::atomic::Ordering::SeqCst);
                crate::rendezvous_mediator::reset_needs_deploy_notification();
                crate::rendezvous_mediator::RendezvousMediator::restart();
            }
            DeployResult::Ok
        }
        "NOT_ENABLED" => DeployResult::NotEnabled,
        "INVALID_INPUT" => DeployResult::InvalidInput,
        "ID_TAKEN" => DeployResult::IdTaken(id_to_deploy),
        _ => {
            if text.is_empty() {
                DeployResult::Error("Unknown response.".to_owned())
            } else {
                DeployResult::Error(text)
            }
        }
    }
}

#[inline]
pub fn has_hwcodec() -> bool {
    // Has real hardware codec using gpu
    (cfg!(feature = "hwcodec") && cfg!(not(target_os = "ios"))) || cfg!(feature = "mediacodec")
}

#[cfg(feature = "dioxus-ui")]
#[inline]
pub fn supported_hwdecodings() -> (bool, bool) {
    let decoding =
        scrap::codec::Decoder::supported_decodings(None, use_texture_render(), None, &vec![]);
    #[allow(unused_mut)]
    let (mut h264, mut h265) = (decoding.ability_h264 > 0, decoding.ability_h265 > 0);
    #[cfg(feature = "vram")]
    {
        // supported_decodings check runtime luid
        let vram = scrap::vram::VRamDecoder::possible_available_without_check();
        if vram.0 {
            h264 = true;
        }
        if vram.1 {
            h265 = true;
        }
    }
    (h264, h265)
}

// Make sure `SENDER` is inited here.
#[inline]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn start_option_status_sync() {
    let _sender = SENDER.lock().unwrap();
}

// not call directly
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn check_connect_status(reconnect: bool) -> mpsc::UnboundedSender<ipc::Data> {
    let (tx, rx) = mpsc::unbounded_channel::<ipc::Data>();
    std::thread::spawn(move || check_connect_status_(reconnect, rx));
    tx
}

// notice: avoiding create ipc connection repeatedly,
// because windows named pipe has serious memory leak issue.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tokio::main(flavor = "current_thread")]
async fn check_connect_status_(reconnect: bool, rx: mpsc::UnboundedReceiver<ipc::Data>) {
    let mut key_confirmed = false;
    let mut rx = rx;
    let mut mouse_time = 0;
    let mut id = "".to_owned();
    let is_cm = crate::common::is_cm();

    loop {
        if let Ok(mut c) = ipc::connect(1000, "").await {
            let mut timer = crate::rustdesk_interval(time::interval(time::Duration::from_secs(1)));
            loop {
                tokio::select! {
                    res = c.next() => {
                        match res {
                            Err(err) => {
                                log::error!("ipc connection closed: {}", err);
                                if is_cm {
                                    crate::ui_cm_interface::quit_cm();
                                }
                                break;
                            }
                            #[cfg(not(any(target_os = "android", target_os = "ios")))]
                            Ok(Some(ipc::Data::MouseMoveTime(v))) => {
                                mouse_time = v;
                                UI_STATUS.lock().unwrap().mouse_time = v;
                            }
                            Ok(Some(ipc::Data::Options(Some(v)))) => {
                                *OPTIONS.lock().unwrap() = v;
                            }
                            Ok(Some(ipc::Data::Config((name, Some(value))))) => {
                                if name == "id" {
                                    {
                                        id = value;
                                    }
                                } else if name == "temporary-password" {
                                    *TEMPORARY_PASSWD.lock().unwrap() = value;
                                }
                            }
                            Ok(Some(ipc::Data::OnlineStatus(Some((mut x, _c))))) => {
                                if x > 0 {
                                    x = 1
                                }
                                {
                                    key_confirmed = _c;
                                }
                                *UI_STATUS.lock().unwrap() = UiStatus {
                                    status_num: x as _,
                                    key_confirmed: _c,
                                    #[cfg(not(any(target_os = "android", target_os = "ios")))]
                                    mouse_time,
                                    id: id.clone(),
                                };
                            }
                            #[cfg(target_os = "windows")]
                            Ok(Some(ipc::Data::FileTransferEnabledState(v))) => {
                                if let Some(enabled) = v {
                                    let mut lock = IS_FILE_TRANSFER_ENABLED.lock().unwrap();
                                    if *lock != v {
                                        clipboard::ContextSend::enable(enabled);
                                        *lock = v;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Some(data) = rx.recv() => {
                        allow_err!(c.send(&data).await);
                    }
                    _ = timer.tick() => {
                        c.send(&ipc::Data::OnlineStatus(None)).await.ok();
                        c.send(&ipc::Data::Options(None)).await.ok();
                        c.send(&ipc::Data::Config(("id".to_owned(), None))).await.ok();
                        c.send(&ipc::Data::Config(("temporary-password".to_owned(), None))).await.ok();
                        #[cfg(target_os = "windows")]
                        c.send(&ipc::Data::FileTransferEnabledState(None)).await.ok();
                    }
                }
            }
        }
        if !reconnect {
            OPTIONS
                .lock()
                .unwrap()
                .insert("ipc-closed".to_owned(), "Y".to_owned());
            break;
        }
        *UI_STATUS.lock().unwrap() = UiStatus {
            status_num: -1,
            key_confirmed,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            mouse_time,
            id: id.clone(),
        };
        sleep(1.).await;
    }
}

const INVALID_FORMAT: &'static str = "Invalid format";
const UNKNOWN_ERROR: &'static str = "Unknown error";

#[inline]
#[tokio::main(flavor = "current_thread")]
pub async fn change_id_shared(id: String, old_id: String) -> String {
    change_id_shared_(id, old_id).await.to_owned()
}

pub async fn change_id_shared_(id: String, old_id: String) -> &'static str {
    if !hbb_common::is_valid_custom_id(&id) {
        log::debug!(
            "debugging invalid id: \"{id}\", len: {}, base64: \"{}\"",
            id.len(),
            crate::encode64(&id)
        );
        let bom = id.trim_start_matches('\u{FEFF}');
        log::debug!("bom: {}", hbb_common::is_valid_custom_id(&bom));
        return INVALID_FORMAT;
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let uuid = Bytes::from(
        hbb_common::machine_uid::get()
            .unwrap_or("".to_owned())
            .as_bytes()
            .to_vec(),
    );
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let uuid = Bytes::from(hbb_common::get_uuid());

    if uuid.is_empty() {
        log::error!("Failed to change id, uuid is_empty");
        return UNKNOWN_ERROR;
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let rendezvous_servers = crate::ipc::get_rendezvous_servers(1_000).await;
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let rendezvous_servers = Config::get_rendezvous_servers();

    let mut futs = Vec::new();
    let err: Arc<Mutex<&str>> = Default::default();
    for rendezvous_server in rendezvous_servers {
        let err = err.clone();
        let id = id.to_owned();
        let uuid = uuid.clone();
        let old_id = old_id.clone();
        futs.push(tokio::spawn(async move {
            let tmp = check_id(rendezvous_server, old_id, id, uuid).await;
            if !tmp.is_empty() {
                *err.lock().unwrap() = tmp;
            }
        }));
    }
    join_all(futs).await;
    let err = *err.lock().unwrap();
    if err.is_empty() {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        crate::ipc::set_config_async("id", id.to_owned()).await.ok();
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            Config::set_key_confirmed(false);
            Config::set_id(&id);
        }
    }
    err
}

async fn check_id(
    rendezvous_server: String,
    old_id: String,
    id: String,
    uuid: Bytes,
) -> &'static str {
    if let Ok(mut socket) = hbb_common::socket_client::connect_tcp(
        crate::check_port(rendezvous_server, RENDEZVOUS_PORT),
        CONNECT_TIMEOUT,
    )
    .await
    {
        let mut msg_out = Message::new();
        msg_out.set_register_pk(RegisterPk {
            old_id,
            id,
            uuid,
            ..Default::default()
        });
        let mut ok = false;
        if socket.send(&msg_out).await.is_ok() {
            if let Some(msg_in) =
                crate::common::get_next_nonkeyexchange_msg(&mut socket, None).await
            {
                match msg_in.union {
                    Some(rendezvous_message::Union::RegisterPkResponse(rpr)) => {
                        match rpr.result.enum_value() {
                            Ok(register_pk_response::Result::OK) => {
                                ok = true;
                            }
                            Ok(register_pk_response::Result::ID_EXISTS) => {
                                return "Not available";
                            }
                            Ok(register_pk_response::Result::TOO_FREQUENT) => {
                                return "Too frequent";
                            }
                            Ok(register_pk_response::Result::NOT_SUPPORT) => {
                                return "server_not_support";
                            }
                            Ok(register_pk_response::Result::SERVER_ERROR) => {
                                return "Server error";
                            }
                            Ok(register_pk_response::Result::INVALID_ID_FORMAT) => {
                                return INVALID_FORMAT;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        if !ok {
            return UNKNOWN_ERROR;
        }
    } else {
        return "Failed to connect to rendezvous server";
    }
    ""
}

// if it's relay id, return id processed, otherwise return original id
pub fn handle_relay_id(id: &str) -> &str {
    if id.ends_with(r"\r") || id.ends_with(r"/r") {
        &id[0..id.len() - 2]
    } else {
        id
    }
}

#[cfg(test)]
mod tests {
    use super::validate_windows_service_video_save_directory;

    #[test]
    fn validate_service_video_save_directory() {
        let absolute = if cfg!(windows) {
            r"C:\recordings"
        } else {
            "/recordings"
        };
        let padded = format!("  {absolute}  ");

        assert_eq!(
            validate_windows_service_video_save_directory(&padded),
            Some(absolute)
        );
        assert_eq!(
            validate_windows_service_video_save_directory("recordings"),
            None
        );
        assert_eq!(
            validate_windows_service_video_save_directory(&format!("\"{absolute}\"")),
            None
        );
        assert_eq!(validate_windows_service_video_save_directory("  "), None);
    }
}
