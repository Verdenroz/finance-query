import os

import numpy
from Cython.Build import cythonize
from setuptools import setup, Extension

# Define the core directory path
CORE_DIR = "src/services/indicators/core"

extensions = [
    Extension(
        "utils",
        [os.path.join(CORE_DIR, "utils.pyx")],
        include_dirs=[numpy.get_include()],
        define_macros=[("NPY_NO_DEPRECATED_API", "NPY_1_7_API_VERSION")]
    ),
    Extension(
        "moving_averages",
        [os.path.join(CORE_DIR, "moving_averages.pyx")],
        include_dirs=[numpy.get_include()],
        define_macros=[("NPY_NO_DEPRECATED_API", "NPY_1_7_API_VERSION")]
    ),
    Extension(
        "oscillators",
        [os.path.join(CORE_DIR, "oscillators.pyx")],
        include_dirs=[numpy.get_include()],
        define_macros=[("NPY_NO_DEPRECATED_API", "NPY_1_7_API_VERSION")]
    ),
    Extension(
        "trends",
        [os.path.join(CORE_DIR, "trends.pyx")],
        include_dirs=[numpy.get_include()],
        define_macros=[("NPY_NO_DEPRECATED_API", "NPY_1_7_API_VERSION")]
    ),
]

setup(
    ext_modules=cythonize(
        extensions,
        compiler_directives={
            'language_level': "3",
            'boundscheck': False,
            'wraparound': False,
            'cdivision': True
        }
    ),
    package_dir={'': 'src/services/indicators/core'},
)