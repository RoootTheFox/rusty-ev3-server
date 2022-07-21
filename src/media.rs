
pub struct MediaState {
    pub playing: bool,
    pub current_song: String,
    pub current_time: u32,
    pub max_time: u32
}

pub fn invert_playing(state:&MediaState) -> MediaState {
    MediaState {
        playing: !state.playing,
        current_song: state.current_song.clone(),
        current_time: state.current_time,
        max_time: state.max_time
    }
}