import sys
import re
import json

def get_trailing_number(s):
    m = re.search(r'\d+$', s)
    return int(m.group()) if m else None

class SongData:
    notes = []
    bpm = 0.0
    song = ""

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
        elif "StartTime:" in line:
            start_time = get_trailing_number(line)
        elif "Lane:" in line:
            lane = get_trailing_number(line)
            song_data.notes.append(Note(start_time, lane))

print("{")
print("\"bpm\": " + str(song_data.bpm) + ",")
print("\"offset\": 0.0,") # you may have to dial this in manually
print("\"song\": \"" + song_data.song.replace("\r", "").replace("\n", "") + "\",")
print("\"notes\": [")
for index, note in enumerate(song_data.notes):
    print("{")
    print("\"lane\": " + str(note.lane) + ",")
    print("\"time\": " + str(note.time/1000))
    if index == len(song_data.notes)-1:
        print("}")
    else:
        print("},")

print("]}")