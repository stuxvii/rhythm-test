import sys
import re
import json

def get_trailing_number(s):
    m = re.search(r"[-+]?\d*\.?\d+|[-+]?\d+", s)
    return float(m.group()) if m else None

class SongData:
    notes = []
    bpm = 0.0
    offset = 0.0
    song = ""
    name = ""

class Note:
    def __init__(self, time, lane):
        self.time = time
        self.lane = lane

song_data = SongData()

with open(sys.argv[1]) as f:
    start_time = 0
    lane = 0
    for line in f:
        if "Bpm" in line:
            song_data.bpm = get_trailing_number(line)
        elif "AudioFile:" in line:
            song_data.song = line.partition("AudioFile: ")[2]
        elif "Title:" in line:
            song_data.name = line.partition("Title: ")[2]
        elif "StartTime:" in line:
            start_time = get_trailing_number(line)
        elif "Lane:" in line:
            lane = get_trailing_number(line)
            song_data.notes.append(Note(start_time, lane))

output_data = {
    "bpm": song_data.bpm,
    "offset": song_data.offset,
    "song": song_data.song.strip(), 
    "name": song_data.name.strip(), 
    "notes": [
        [int(n.lane),n.time/1000] for n in song_data.notes
    ]
}

print(json.dumps(output_data, separators=(',', ':')))

# must pipe this output to file manually in the terminal!