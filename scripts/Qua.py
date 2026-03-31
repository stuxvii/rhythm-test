import sys
import re
import json

def get_trailing_number(s):
    m = re.search(r"[-+]?\d*\.?\d+|[-+]?\d+", s) 
    return float(m.group()) if m else 0.0

class SongData:
    def __init__(self):
        self.notes = []
        self.bpm = 0.0
        self.offset = 0.0
        self.song = ""
        self.name = ""

class Note:
    def __init__(self, time, lane, end_time):
        self.time = time
        self.lane = lane
        self.end_time = end_time

song_data = SongData()

with open(sys.argv[1]) as f:
    current_note = None
    going_thru_hitobjects = False

    for line in f:
        line = line.strip()
        
        if "Bpm:" in line:
            song_data.bpm = get_trailing_number(line)
        elif "AudioFile:" in line:
            song_data.song = line.partition("AudioFile:")[2].strip()
        elif "Title:" in line:
            song_data.name = line.partition("Title:")[2].strip()
        elif "HitObjects:" in line:
            going_thru_hitobjects = True
            continue

        if going_thru_hitobjects:
            if line.startswith("-"):
                if current_note:
                    song_data.notes.append(current_note)
                val = get_trailing_number(line)
                current_note = Note(val, 0, 0) 
            
            elif "Lane:" in line and current_note:
                current_note.lane = int(get_trailing_number(line))
            elif "EndTime:" in line and current_note:
                current_note.end_time = get_trailing_number(line)

    if current_note:
        song_data.notes.append(current_note)

output_data = {
    "bpm": song_data.bpm,
    "offset": song_data.offset,
    "song": song_data.song, 
    "name": song_data.name, 
    "notes": [
        [int(n.lane), n.time/1000, n.end_time/1000] for n in song_data.notes
    ]
}

print(json.dumps(output_data, separators=(',', ':')))