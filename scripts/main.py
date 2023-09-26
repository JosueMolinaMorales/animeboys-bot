import json



def find_unique_keys(json_obj, parent_key='', separator='.'):
    keys = []
    if isinstance(json_obj, dict):
        for key, value in json_obj.items():
            new_key = f"{parent_key}{separator}{key}" if parent_key else key
            keys.append(new_key)
            keys.extend(find_unique_keys(value, new_key, separator))
    elif isinstance(json_obj, list):
        for item in json_obj:
            new_key = f"{parent_key}"
            keys.append(new_key)
            keys.extend(find_unique_keys(item, new_key, separator))
    return keys


# Example JSON object
with open("output.json", 'r') as f:
    json_obj = json.load(f)
    # Find and print unique keys

    keys = find_unique_keys(json_obj['builds'])
    keys.sort()
    unique_keys = set(keys)
    for key in unique_keys:
        if str(key).count('.') == 0:
            print(key)
