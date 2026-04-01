import sys
import yaml
import json

def parse_song_data(file_path):
    with open(file_path, 'r') as f:
        data = yaml.safe_load(f)

    bpm = 0.0
    if "TimingPoints" in data and isinstance(data["TimingPoints"], list):
        bpm = float(data["TimingPoints"][0].get("Bpm", 0.0))

    processed_notes = []
    hit_objects = data.get("HitObjects", [])
    
    if isinstance(hit_objects, list):
        for obj in hit_objects:
            lane = int(obj.get("Lane", 0))
            time = float(obj.get("StartTime", 0)) / 1000
            end_time = float(obj.get("EndTime", 0)) / 1000
            
            processed_notes.append([lane, time, end_time])

    output_data = {
        "bpm": bpm,
        "offset": 0.0,
        "song": data.get("AudioFile", ""),
        "name": data.get("Title", ""),
        "notes": processed_notes
    }

    return output_data

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python Qua.py <filename>")
        sys.exit(1)

    result = parse_song_data(sys.argv[1])
    
    print(json.dumps(result, separators=(',', ':')))