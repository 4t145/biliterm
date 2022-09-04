use bilibili_client::ClientError;
#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    ConnectLiveRoomFail,
    WebApiClientFail(ClientError),
    Io(std::io::Error)
}