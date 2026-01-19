import os

os.chdir("C:/Users/micha/snappa_survivors_redux/game/assets")

survivors = ["dewey", "matthew", "mark", "shaunt", "paul", "ryan", "gabe", "finn"]

for survivor in survivors:
    file_name = "survivors/" + survivor + "/stats.ron"
    with open(file_name, "w") as file:
        file.write("([Health((max: 50.0, current: 50.0))])")
