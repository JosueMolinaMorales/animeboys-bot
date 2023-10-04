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

def find_all_weapon_ids(builds):
    weapon_ids = []
    for build in builds:
        weapon_ids.append(build['weaponId'])

    # Remove all duplicates
    weapon_ids = list(set(weapon_ids))
    return weapon_ids

def main():
    # Display the menu
    print("=============================================")
    print("1. Find all unique keys")
    print("2. Find all weapon IDs")
    print("=============================================")
    print("Enter your choice: ")
    choice = input()

    # Load json object
    json_obj = json.load(open("output.json", 'r'))
    if choice == '1':
        keys = find_unique_keys(json_obj['builds'])
        keys.sort()
        unique_keys = set(keys)
        for key in unique_keys:
            if str(key).count('.') == 0:
                print(key)
    elif choice == '2':
        weapon_ids = find_all_weapon_ids(json_obj['builds'])
        weapon_ids.sort()
        for weapon_id in weapon_ids:
            print(weapon_id)



if __name__ == '__main__':
    main()