import cv2
import numpy as np
import tkinter as tk
from PIL import Image, ImageTk

from distortion import reverse_radial_distortion


class DistortionTuner:

    def __init__(self, image_path):

        self.original = cv2.imread(image_path)

        if self.original is None:
            raise FileNotFoundError(
                f"Could not load image: {image_path}"
            )

        self.height, self.width = self.original.shape[:2]

        self.root = tk.Tk()
        self.root.title(
            "Fallout Terminal Radial Distortion Tuner"
        )

        # -------------------------------------------------
        # Image preview
        # -------------------------------------------------

        self.preview = tk.Label(self.root)
        self.preview.pack(
            padx=10,
            pady=10
        )

        # -------------------------------------------------
        # Controls
        # -------------------------------------------------

        controls = tk.Frame(self.root)
        controls.pack(
            fill="x",
            padx=10
        )

        self.variables = {}

        # Distortion parameters
        self.add_slider(
            controls,
            "k1",
            -0.30,
            0.10,
            0.0,
            0.001
        )
        self.add_slider(
            controls,
            "k2",
            -0.10,
            0.10,
            0.0,
            0.001
        )

        self.add_slider(
            controls,
            "center_x",
            self.width * 0.35,
            self.width * 0.65,
            self.width / 2,
            1.0
        )

        self.add_slider(
            controls,
            "center_y",
            self.height * 0.20,
            self.height * 0.70,
            self.height / 2,
            1.0
        )

        self.add_slider(
            controls,
            "fx",
            0,
            self.width,
            self.width / 2,
            1.0
        )

        self.add_slider(
            controls,
            "fy",
            0,
            self.height,
            self.height / 2,
            1.0
        )

        # -------------------------------------------------
        # Grid parameters
        # -------------------------------------------------

        self.add_slider(
            controls,
            "grid_x_stride",
            5,
            200,
            30,
            1.0
        )

        self.add_slider(
            controls,
            "grid_y_stride",
            5,
            200,
            60,
            1.0
        )

        self.add_slider(
            controls,
            "grid_x_offset",
            0,
            200,
            0,
            1.0
        )

        self.add_slider(
            controls,
            "grid_y_offset",
            0,
            200,
            0,
            1.0
        )

        # -------------------------------------------------
        # Grid enable checkbox
        # -------------------------------------------------

        self.grid_enabled = tk.BooleanVar(
            value=True
        )

        tk.Checkbutton(
            controls,
            text="Show grid",
            variable=self.grid_enabled,
            command=self.update
        ).pack(
            pady=5
        )

        # -------------------------------------------------
        # Buttons
        # -------------------------------------------------

        buttons = tk.Frame(self.root)
        buttons.pack(pady=8)

        tk.Button(
            buttons,
            text="Save corrected image",
            command=self.save_image
        ).pack(
            side="left",
            padx=5
        )

        tk.Button(
            buttons,
            text="Print parameters",
            command=self.print_parameters
        ).pack(
            side="left",
            padx=5
        )

        tk.Button(
            buttons,
            text="Reset",
            command=self.reset
        ).pack(
            side="left",
            padx=5
        )

        # Initial image.
        self.update()

    # =====================================================
    # Slider creation
    # =====================================================

    def add_slider(
        self,
        parent,
        name,
        minimum,
        maximum,
        initial,
        resolution
    ):

        frame = tk.Frame(parent)
        frame.pack(
            fill="x"
        )

        tk.Label(
            frame,
            text=name,
            width=16,
            anchor="w"
        ).pack(
            side="left"
        )

        variable = tk.DoubleVar(
            value=initial
        )

        self.variables[name] = variable

        scale = tk.Scale(
            frame,
            from_=minimum,
            to=maximum,
            resolution=resolution,
            orient="horizontal",
            variable=variable,
            command=lambda value: self.update(),
            length=600,
        )

        scale.pack(
            side="left",
            fill="x",
            expand=True
        )

        value_label = tk.Label(
            frame,
            width=10
        )

        value_label.pack(
            side="left"
        )

        def update_label(*args):
            value_label.config(
                text=f"{variable.get():.3f}"
            )

        variable.trace_add(
            "write",
            update_label
        )

        update_label()

    # =====================================================
    # Get parameters
    # =====================================================

    def get_parameters(self):

        return {
            name: variable.get()
            for name, variable in self.variables.items()
        }

    # =====================================================
    # Draw grid
    # =====================================================

    def draw_grid(self, image):

        p = self.get_parameters()

        x_stride = int(p["grid_x_stride"])
        y_stride = int(p["grid_y_stride"])

        x_offset = int(p["grid_x_offset"])
        y_offset = int(p["grid_y_offset"])

        # Make sure strides are valid.
        if x_stride <= 0 or y_stride <= 0:
            return image

        # Draw vertical lines.
        for x in range(
            x_offset,
            self.width,
            x_stride
        ):
            cv2.line(
                image,
                (x, 0),
                (x, self.height - 1),
                (0, 0, 255),
                1,
                cv2.LINE_AA
            )

        # Draw horizontal lines.
        for y in range(
            y_offset,
            self.height,
            y_stride
        ):
            cv2.line(
                image,
                (0, y),
                (self.width - 1, y),
                (0, 0, 255),
                1,
                cv2.LINE_AA
            )

        return image

    # =====================================================
    # Update image
    # =====================================================

    def update(self):

        p = self.get_parameters()

        corrected = reverse_radial_distortion(
            self.original,

            k1=p["k1"],
            k2=p["k2"],

            center_x=p["center_x"],
            center_y=p["center_y"],

            fx=p["fx"],
            fy=p["fy"],
        )

        # -------------------------------------------------
        # Overlay grid AFTER distortion correction.
        # -------------------------------------------------

        if self.grid_enabled.get():
            corrected = self.draw_grid(
                corrected
            )

        # -------------------------------------------------
        # Scale image for display.
        # -------------------------------------------------

        max_width = 1200
        max_height = 750

        scale = min(
            max_width / self.width,
            max_height / self.height,
            1.0
        )

        display_width = int(
            self.width * scale
        )

        display_height = int(
            self.height * scale
        )

        display = cv2.resize(
            corrected,
            (
                display_width,
                display_height
            ),
            interpolation=cv2.INTER_AREA
        )

        display = cv2.cvtColor(
            display,
            cv2.COLOR_BGR2RGB
        )

        photo = ImageTk.PhotoImage(
            Image.fromarray(display)
        )

        self.preview.configure(
            image=photo
        )

        self.preview.image = photo

    # =====================================================
    # Print parameters
    # =====================================================

    def print_parameters(self):

        p = self.get_parameters()

        print()
        print()
        print("Current distortion parameters:")
        print(
            f"k1={p['k1']:.3f}"
        )
        print(
            f"k2={p['k2']:.3f}"
        )
        print(
            f"center_x={p['center_x']:.1f}"
        )
        print(
            f"center_y={p['center_y']:.1f}"
        )
        print(
            f"fx={p['fx']:.1f}"
        )
        print(
            f"fy={p['fy']:.1f}"
        )

        print()
        print("Grid parameters:")
        print(
            f"grid_x_stride={int(p['grid_x_stride'])}"
        )
        print(
            f"grid_y_stride={int(p['grid_y_stride'])}"
        )
        print(
            f"grid_x_offset={int(p['grid_x_offset'])}"
        )
        print(
            f"grid_y_offset={int(p['grid_y_offset'])}"
        )
        print()

    # =====================================================
    # Save
    # =====================================================

    def save_image(self):

        p = self.get_parameters()

        corrected = reverse_radial_distortion(
            self.original,

            k1=p["k1"],
            k2=p["k2"],

            center_x=p["center_x"],
            center_y=p["center_y"],

            fx=p["fx"],
            fy=p["fy"],
        )

        # Do NOT save the grid. The saved image is intended
        # for the actual template-matching pipeline.
        cv2.imwrite(
            "corrected2.png",
            corrected
        )

        print()
        print("Saved corrected.png")

        self.print_parameters()

    # =====================================================
    # Reset
    # =====================================================

    def reset(self):

        defaults = {
            "k1": -0.027,
            "k2": -0.014,
            "center_x": 669.0,
            "center_y": 410.0,
            "fx": self.width / 2,
            "fy": self.height / 2,
            "grid_x_stride": 22,
            "grid_y_stride": 43,
            "grid_x_offset": 25,
            "grid_y_offset": 88,
        }
        defaults = {
            "k1": -0.018,
            "k2": 0.0,
            "center_x": 669.0,
            "center_y": 410.0,
            "fx": 336,
            "fy": 328,
            "grid_x_stride": 22,
            "grid_y_stride": 43,
            "grid_x_offset": 25,
            "grid_y_offset": 88,
        }
        defaults = {
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

        for name, value in defaults.items():
            self.variables[name].set(value)

        self.grid_enabled.set(True)

        self.update()


# =========================================================
# Main
# =========================================================

if __name__ == "__main__":

    import sys

    if len(sys.argv) != 2:
        print(
            "Usage:"
        )
        print(
            "    python tuner.py minigame.png"
        )
        raise SystemExit(1)

    tuner = DistortionTuner(
        sys.argv[1]
    )

    tuner.root.mainloop()#!/usr/bin/env python3
