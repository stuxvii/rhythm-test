## RHYTHM GAME CLIENT
hi!!! this is my very kewl rhythm game raylib thingy hope you liek it :]
## MAP FORMAT
Maps are formatted as follows:
```json
{
    "bpm": Float.
    "offset": Float. Where to begin audio playback.
    "song": String. The path to the audio file corresponding to the song.
    "notes": List. A list of notes.
}
```
Notes are formatted as so:
```json
[1,40.227],
[2,40.532],
[4,40.684]
```
The first value corresponds to which lane the note is in.<br>
The second value corresponds to how many seconds the note appears at.<br>
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

Maps must be packaged

## CONVERTS
If you wish to convert maps from your preferred rhythm game, commodity scripts will be present at ``./scripts/``.<br>
At the moment, only a Quaver map converter helper is provided, but support for other games such as Osu! and StepMania are planned.