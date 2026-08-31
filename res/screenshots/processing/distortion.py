import cv2
import numpy as np

def reverse_crt_distortion(
    image,
    bow_y=18.0,
    center_x=695.0,
    tilt_y=-2.4,
    bow_x=0.0,
    center_y=None,
    interpolation=cv2.INTER_LINEAR,
):
    """
    Reverse the curved-screen distortion found in Fallout 4 terminals.

    Parameters
    ----------
    image:
        OpenCV image (BGR).

    bow_y:
        Vertical correction at the left/right edges, in pixels.
        Positive values straighten a screen whose horizontal lines
        curve downward toward the edges.

    center_x:
        X coordinate of the center of the screen's curvature.

    tilt_y:
        Additional linear vertical correction across the image.
        Useful for compensating for a slight overall tilt.

    bow_x:
        Horizontal correction at the top/bottom edges, in pixels.
        Usually 0 for the Fallout 4 terminals shown here.

    center_y:
        Center of the vertical curvature. Defaults to image center.

    interpolation:
        OpenCV interpolation method. INTER_LINEAR is a good default.
    """

    h, w = image.shape[:2]

    if center_y is None:
        center_y = h / 2.0

    x = np.arange(w, dtype=np.float32)
    y = np.arange(h, dtype=np.float32)

    X, Y = np.meshgrid(x, y)

    # Normalized distance from the curvature centers.
    nx = (X - center_x) / (w / 2.0)
    ny = (Y - center_y) / (h / 2.0)

    # Reverse the horizontal screen curvature.
    map_y = Y + bow_y * (nx ** 2)

    # Optional vertical curvature.
    map_x = X + bow_x * (ny ** 2)

    # Correct the slight overall tilt.
    map_y += tilt_y * (X / (w - 1) - 0.5)

    corrected = cv2.remap(
        image,
        map_x,
        map_y,
        interpolation,
        borderMode=cv2.BORDER_CONSTANT,
        borderValue=(0, 0, 0),
    )

    return corrected


def reverse_radial_distortion(
    image,
    k1=-0.08,
    k2=0.0,
    center_x=None,
    center_y=None,
    fx=None,
    fy=None,
    interpolation=cv2.INTER_LINEAR,
):
    """
    Reverse radial barrel distortion.

    Parameters
    ----------
    image : numpy.ndarray
        Input OpenCV image.

    k1 : float
        Primary radial distortion coefficient.

        Negative values compensate for barrel distortion.

    k2 : float
        Secondary radial distortion coefficient.

        Usually this can remain 0 unless k1 alone cannot accurately
        describe the distortion.

    center_x : float
        X coordinate of the distortion center.

    center_y : float
        Y coordinate of the distortion center.

        This does NOT necessarily need to be the center of the image.

    fx : float
        Horizontal normalization scale.

    fy : float
        Vertical normalization scale.

    interpolation : int
        OpenCV interpolation method.

    Returns
    -------
    numpy.ndarray
        Distortion-corrected image.
    """

    height, width = image.shape[:2]

    if center_x is None:
        center_x = width / 2

    if center_y is None:
        center_y = height / 2

    if fx is None:
        fx = width / 2

    if fy is None:
        fy = height / 2

    # Pixel coordinate grid.
    y, x = np.indices(
        (height, width),
        dtype=np.float32
    )

    # Convert to normalized coordinates around the
    # distortion center.
    xn = (x - center_x) / fx
    yn = (y - center_y) / fy

    r2 = xn * xn + yn * yn

    # Radial distortion factor.
    factor = 1.0 + k1 * r2 + k2 * r2 * r2

    # Map each pixel in the corrected image back into
    # the original distorted image.
    map_x = center_x + xn * factor * fx
    map_y = center_y + yn * factor * fy

    map_x = map_x.astype(np.float32)
    map_y = map_y.astype(np.float32)

    corrected = cv2.remap(
        image,
        map_x,
        map_y,
        interpolation,
        borderMode=cv2.BORDER_CONSTANT,
        borderValue=(0, 0, 0),
    )

    return corrected

