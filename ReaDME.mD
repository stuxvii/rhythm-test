## RHYTHM GAME SHIT FUCK BUILT ON RUST USING THE MACROQUAD LIBRARY

MAP FORMAT & EXAMPLE:

Each newline after the start command indicates a bar. As in musical bars.
<br>
[dfjk] are the usual keybinds for rhythm games:
<br>
d: first lane
<br>
f: second lane
<br>
j: third lane
<br>
k: fourth lane
<br>
<br>
You can put them in whatever order you want, kjfd is the same as dfjk.
<br>
<br>
Barebones mid-song BPM changes are implemented, little buggy. Submit a pull request.
You can use them like this:
``dfjk 300`` or like this: ```. 300``` <- the dot here can be any character but dfjk. just make sure there's a space.
<br>
```
bpm 70
start
dfjk
d
kdf

 300

f
j
k
```

<br>
NOTE: Only free audio formats like FLAC, OGG and WAV are supported, and depending on your OS and audio server it may take longer or shorter to load the audio. Blame audrey.