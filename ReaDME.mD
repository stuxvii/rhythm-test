## RHYTHM GAME CLIENT
hi!!! this is my very kewl rhythm game raylib thingy hope you liek it :]
## CONFIGURATION
You will find 5 keys in config.json:
```json
{
    "scroll_speed": Number, Self explanatory,
    "max_fps": Number, Maximum amount of frames per second,
    "hitsound": String, Path to the sound that will be played when a key is pressed, relative to the executable,
    "autoplay": If this is set to true, then when a song is loaded it will be visualized,
    "quit_after_song_end": If this is set to true, then when a song is over, the game will quit after 5 seconds,
}
```
## MAP FORMAT
Maps are formatted as follows:
```json
{
    "bpm": Number.
    "offset": Number. Where to begin audio playback.
    "song": String. The path to the audio file corresponding to the song.
    "notes": List. A list of notes.
}
```
Notes are formatted as so:
```json
[1,40.227],
[2,40.532],
[4,40.684, 40.956]
```
The first value corresponds to which lane the note is in.<br>
The second value corresponds to the time the note appears at.<br>
The third value corresponds to the time the note should stop being held. It is completely optional.<br>
Example map:
```json
{
    "bpm": 197,
    "offset": 39.0,
    "song": "cool_song.mp3",
    "notes": [
        [1,40.227],
        [2,40.532],
        [4,40.684],
        [4,40.836],
        [3,40.989]
    ]
}
```


## CONVERTS
If you wish to convert maps from your preferred rhythm game, commodity scripts will be present at ``./scripts/``.<br>
At the moment, only a Quaver map converter helper is provided, but support for other games such as Osu! and StepMania are planned.