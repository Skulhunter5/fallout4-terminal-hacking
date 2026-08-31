import cv2
import numpy as np
import pytesseract


def create_text_mask(image):
    """
    Isolate Fallout terminal's green text from the dark background.

    The text is considerably greener than the surrounding pixels, so
    comparing the green channel against the red/blue channels works
    better than ordinary grayscale thresholding.
    """

    b, g, r = cv2.split(image)

    # How much greener is this pixel than the other channels?
    green_score = (
        g.astype(np.float32)
        - (r.astype(np.float32) + b.astype(np.float32)) / 2
    )

    # Keep pixels that are sufficiently green.
    mask = np.where(green_score > 40, 255, 0).astype(np.uint8)

    return mask


def find_selected_region(image):
    """
    Detect the bright-green selection bar used by Fallout terminals.

    Returns (x, y, width, height), or None if no selection bar is found.
    """

    hsv = cv2.cvtColor(image, cv2.COLOR_BGR2HSV)

    h, s, v = cv2.split(hsv)

    # Bright, saturated green.
    mask = (
        (v > 150)
        & (s > 150)
        & (h > 40)
        & (h < 90)
    ).astype(np.uint8) * 255

    num_labels, labels, stats, _ = cv2.connectedComponentsWithStats(mask)

    best = None
    best_area = 0

    for i in range(1, num_labels):
        x, y, w, h, area = stats[i]

        # Selection bars are very wide and relatively short.
        if area > 10000 and w > 500 and w / max(h, 1) > 5:
            if area > best_area:
                best_area = area
                best = (x, y, w, h)

    return best


def read_selected_line(image, region):
    """
    OCR a selected terminal line.

    Selected text is dark on a bright-green background, so the normal
    green-text mask cannot see it. We instead threshold for dark pixels.
    """

    x, y, w, h = region

    # Small padding around the selection bar.
    y1 = max(0, y - 3)
    y2 = min(image.shape[0], y + h + 3)

    roi = image[y1:y2, x:x + w]

    gray = cv2.cvtColor(roi, cv2.COLOR_BGR2GRAY)

    # Dark characters become white.
    text_mask = np.where(gray < 120, 255, 0).astype(np.uint8)

    # Upscale for OCR.
    text_mask = cv2.resize(
        text_mask,
        None,
        fx=3,
        fy=3,
        interpolation=cv2.INTER_CUBIC
    )

    text = pytesseract.image_to_string(
        text_mask,
        config="--psm 7"
    ).strip()

    return text


def read_terminal_text(image_path):
    image = cv2.imread(image_path)

    if image is None:
        raise FileNotFoundError(
            f"Could not load image: {image_path}"
        )

    # -----------------------------------------
    # Normal green text
    # -----------------------------------------

    mask = create_text_mask(image)

    # Find the selected line.
    selected_region = find_selected_region(image)

    if selected_region is not None:
        x, y, w, h = selected_region

        # Remove the selected region from the normal OCR image.
        # Its text is dark, so it shouldn't be processed by the
        # normal green-text pipeline anyway.
        mask[y:y + h, x:x + w] = 0

    # Upscale.
    mask = cv2.resize(
        mask,
        None,
        fx=3,
        fy=3,
        interpolation=cv2.INTER_CUBIC
    )

    normal_text = pytesseract.image_to_string(
        mask,
        config="--psm 6"
    ).strip()

    # -----------------------------------------
    # Selected line
    # -----------------------------------------

    selected_text = None

    if selected_region is not None:
        selected_text = read_selected_line(
            image,
            selected_region
        )

    # -----------------------------------------
    # Output
    # -----------------------------------------

    if selected_text:
        print("Selected line detected:")
        print(selected_text)
        print()

    print("Normal OCR:")
    print(normal_text)

    return normal_text, selected_text


if __name__ == "__main__":
    read_terminal_text("terminal.png")
