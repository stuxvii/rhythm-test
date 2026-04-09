## ACIDBOX93's QUAVER CHART PLAYER
Hi, this is my player for quaver charts, I made this to be able to play Quaver maps on the go without needing to be connected online or have steam open. Also to learn how to make rhythm games.
<br>
It is extremely light. When compiled, the executable size will be less than 3 megabytes and have all necessary dependencies bundled inside. Its ram usage is particularly low. Achieving less than 100MB under gameplay.

## CONFIGURATION
You will find 5 configs in config.json:
```json
{
    "scroll_speed": Number, Self explanatory,
    "max_fps": Number, Maximum amount of frames per second,
    "hitsound": String, Path to the sound that will be played when a key is pressed, relative to the executable,
    "autoplay": If this is set to true, then when a song is loaded it will be visualized,
    "quit_after_song_end": If this is set to true, then when a song is over, the game will quit after 5 seconds,
    "lane_x_key": Number, this is a keycode. Up to 7 lanes are supported.
}
```
 You may obtain a list of keycodes inside the [documentation for raylib's Rust bindings.](https://docs.rs/raylib/latest/raylib/consts/enum.KeyboardKey.html)