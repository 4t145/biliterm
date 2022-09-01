use bilibili_client::ClientError;

#[derive(Debug)]
pub enum Error {
    ConnectLiveRoomFail,
    WebApiClientFail(ClientError),
    Io(std::io::Error)
}