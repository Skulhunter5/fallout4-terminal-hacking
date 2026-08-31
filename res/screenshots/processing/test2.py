import cv2 as cv
import numpy as np
from matplotlib import pyplot as plt
import os

from distortion import reverse_radial_distortion

def group_into_bins(coords, n_bins):
    y_coords = [coord[0] for coord in coords]
    lo, hi = min(y_coords), max(y_coords)
    width = (hi - lo) / n_bins

    bins = [[] for _ in range(n_bins)]

    for coord in coords:
        i = min(int((coord[1] - lo) / width), n_bins - 1)
        bins[i].append(coord)

    return bins

def group_by_distance(items, threshold=20, key=lambda x: x):
    items = sorted(items, key=key)

    if not items:
        return []

    groups = [[items[0]]]

    for item in items[1:]:
        if key(item) - key(groups[-1][-1]) <= threshold:
            groups[-1].append(item)
        else:
            groups.append([item])

    return groups

def deduplicate_by_distance(items, threshold=7, key=lambda x: x, value=lambda x: x):
    items = sorted(items, key=key)

    if not items:
        return []

    result = [items[0]]

    for item in items[1:]:
        if key(item) - key(result[-1]) > threshold:
            result.append(item)
        elif value(item) > value(result[-1]):
            result[-1] = item

    return result

def find_points(img_gray, template, threshold=0.8):
    w, h = template.shape[::-1]
     
    res = cv.matchTemplate(img_gray,template,cv.TM_CCOEFF_NORMED)
    loc = np.where( res >= threshold)
    points = list(zip(*loc[::-1]))
    points = [(int(p[0]), int(p[1]), w, h, float(res[p[1]][p[0]])) for p in points]
    # for pt in zip(*loc[::-1]):

    return points

def lines_to_grid(lines, x_threshold=10):
    # Collect all x positions from all points
    xs = sorted(p[0] for line in lines for p in line)

    # Cluster x positions into columns
    columns = []
    for x in xs:
        if not columns or x - columns[-1] > x_threshold:
            columns.append(float(x))
        else:
            # Update column center
            columns[-1] = (columns[-1] + x) / 2

    # Create empty grid
    grid = np.full(
        (len(lines), len(columns)),
        None,
        dtype=object
    )

    # Assign each point to its nearest column
    for row, line in enumerate(lines):
        for p in line:
            x = p[0]
            col = min(
                range(len(columns)),
                key=lambda i: abs(columns[i] - x)
            )
            grid[row, col] = p[5]

    return grid

def lines_to_grid2(lines, x_stride=20, x_tolerance=10):
    """
    Convert rows of noisy 2D points into a sparse grid.

    Each point is expected to have:
        p[0] = x
        p[1] = y
        p[5] = label

    `lines` are already grouped into rows and sorted left-to-right.
    """

    # All observed x coordinates
    xs = [p[0] for line in lines for p in line]

    if not xs:
        return np.empty((0, 0), dtype=object)

    # Find a reasonable origin for the grid.
    #
    # We don't know exactly where column 0 is, so try each observed
    # x position as the origin and choose the one that gives the
    # smallest overall distance to integer multiples of x_stride.
    best_origin = None
    best_error = float("inf")

    for origin in xs:
        error = sum(
            min(
                abs(x - (origin + round((x - origin) / x_stride) * x_stride)),
                x_tolerance
            )
            for x in xs
        )

        if error < best_error:
            best_error = error
            best_origin = origin

    origin = best_origin

    # Convert every x coordinate to a discrete column.
    def x_to_col(x):
        return round((x - origin) / x_stride)

    # Determine the required grid width.
    columns = [x_to_col(x) for x in xs]
    min_col = min(columns)
    max_col = max(columns)

    n_cols = max_col - min_col + 1

    grid = np.full(
        (len(lines), n_cols),
        None,
        dtype=object
    )

    # Populate the grid.
    for row, line in enumerate(lines):
        for p in line:
            col = x_to_col(p[0]) - min_col

            # Optional sanity check: reject points that don't fit
            # reasonably close to the inferred grid position.
            expected_x = origin + (col + min_col) * x_stride

            if abs(p[0] - expected_x) <= x_tolerance:
                grid[row, col] = p[5]

    return grid

def print_grid(grid, empty_cell=".", column_separator=""):
    for (i, row) in enumerate(grid):
        print("{:02d}:".format(i), column_separator.join(empty_cell if x is None else str(x) for x in row))

def grid_to_string_lines(grid):
    lines = []
    for (i, row) in enumerate(grid):
        line = "".join(" " if x is None else str(x) for x in row)
        lines.append(line.rstrip())
    return lines

def print_grid_string(grid_string):
    for line in grid_string:
        print(line)

def get_accuracy(a, b):
    total = 0
    correct = 0
    incorrect = []
    for (y, (la, lb)) in enumerate(zip(a, b)):
        for (x, (ca, cb)) in enumerate(zip(la, lb)):
            total = total + 1
            if ca == cb:
                correct = correct + 1
            else:
                incorrect.append((x, y, ca, cb))
    return (correct / total, correct, total, incorrect)

def label_to_color(label):
    label2color = {
        "(": (0, 0, 255),
        ")": (0, 0, 255),
        "[": (255, 0, 0),
        "]": (255, 0, 0),
        "{": (255, 0, 255),
        "}": (255, 0, 255),
        "|": (255, 255, 0),
    }
    default_color = (0, 255, 255)
    color = label2color.get(label)
    return default_color if color is None else color

def char_to_color(char, line):
    (x, y, w, h, val, label) = char
    return [(255, 0, 0), (0, 0, 255)][line % 2]










 
img_original = cv.imread('minigame.png')
assert img_original is not None, "file could not be read, check with os.path.exists()"
p = {
    "k1": -0.021,
    "k2": 0.0,
    "center_x": 669.0,
    "center_y": 410.0,
    "fx": 332,
    "fy": 350,
    "grid_x_stride": 22,
    "grid_y_stride": 43,
    "grid_x_offset": 25,
    "grid_y_offset": 88,
}
img_rgb = reverse_radial_distortion(
    img_original,

    k1=p["k1"],
    k2=p["k2"],

    center_x=p["center_x"],
    center_y=p["center_y"],

    fx=p["fx"],
    fy=p["fy"],
)
img_gray = cv.cvtColor(img_rgb, cv.COLOR_BGR2GRAY)



THRESHOLD_MATCH=0.8
THRESHOLD_X=6
THRESHOLD_Y=10
THRESHOLD_GRID_X=8



chars = []
for filename in os.listdir("./chars/"):
# for filename in ["{.png", "}.png", "(.png", ").png", "[.png", "].png", "|.png"]:
    if not filename.endswith(".png"):
        continue
    char = filename[:-len(".png")]
    # if char == "^":
    #     continue
    if char == "slash":
        char = "/"
    elif char == "minus":
        char = "-"
    elif char.startswith("period"):
        char = "."
    elif char.startswith("gravis"):
        char = "`"
    print("matching", char, end="")
    template = cv.imread("./chars/" + filename, cv.IMREAD_GRAYSCALE)
    assert template is not None, "file could not be read, check with os.path.exists()"
    threshold = THRESHOLD_MATCH
    if char == "@":
        threshold = 0.75
    elif char == "n":
        threshold = 0.85
    elif char == "^":
        threshold = 0.9
    elif char == ",":
        threshold = 0.9
    elif char == ".":
        threshold = 0.92
    elif char == "_":
        threshold = 0.93
    elif char == "'":
        threshold = 0.85
    elif char == "`":
        threshold = 0.85
    elif char in [";", ":"]:
        threshold = 0.83
    points = find_points(img_gray, template, threshold=threshold)
    print("", len(points))
    chars.extend([(p[0], p[1], p[2], p[3], p[4], char) for p in points])


print()


lines = group_by_distance(chars, threshold=THRESHOLD_Y, key=lambda x: x[1])
for i in range(len(lines)):
    lines[i] = deduplicate_by_distance(lines[i], key=lambda x: x[0], value=lambda x: x[4], threshold=THRESHOLD_X)
print("lines", len(lines))
for (i, line) in enumerate(lines):
    # print(line[0][1], len(line), [(c[0], c[5]) for c in line])
    for char in line:
        (px, py, pw, ph, pval, plabel) = char
        # color = label_to_color(plabel)
        color = char_to_color(char, i)
        cv.rectangle(img_rgb, (px, py), (px + pw, py + ph), color, 2)

cv.imwrite('res.png', img_rgb)


print()


# grid = lines_to_grid(lines, x_threshold=THRESHOLD_GRID_X)
grid = lines_to_grid2(lines, x_stride=22, x_tolerance=10)
print_grid(grid, empty_cell=" ", column_separator="")


print()


string_lines = grid_to_string_lines(grid)
truth = []
with open("minigame.txt", "r") as file:
    for line in file:
        truth.append(line.rstrip())

(accuracy, correct, total, incorrect) = get_accuracy(string_lines, truth)
print("accuracy", "{:.2f}%".format(accuracy * 100), "({:d}/{:d})".format(correct, total))
print()
print(incorrect)
