import os

import numpy
from Cython.Build import cythonize
from setuptools import Extension, setup

CORE_DIR = "src/services/indicators/core"

module_names = ["utils", "moving_averages", "oscillators", "trends"]


extensions = [
    Extension(
        f"services.indicators.core.{name}",
        [os.path.join(CORE_DIR, f"{name}.pyx")],
        include_dirs=[numpy.get_include()],
        define_macros=[("NPY_NO_DEPRECATED_API", "NPY_1_7_API_VERSION")],
    )
    for name in module_names
]

setup(
    ext_modules=cythonize(
        extensions,
        compiler_directives={"language_level": "3", "boundscheck": False, "wraparound": False, "cdivision": True},
    ),
)
