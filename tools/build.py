from __future__ import print_function

import os
import sys
import zipfile
import subprocess
import shutil
import platform
import tempfile
import argparse
import json
import tarfile
import time
import re

try:
    from urllib.request import urlopen, urlretrieve, Request
    from urllib.error import HTTPError
except ImportError:
    from urllib import urlretrieve
    from urllib2 import urlopen, Request

VERBOSE = ''
MESSAGE = '\033[94m'
SUCCESS = '\033[92m'
WARNING = '\033[93m'
ERROR = '\033[91m'
ENDC = '\033[0m'

def log(level, message):
  if os.environ.get('CARGO') != None:
    if (level == WARNING or level == ERROR):
      print('cargo:warning=[{} v{}] {}'.format(os.environ.get('CARGO_PKG_NAME'), os.environ.get('CARGO_PKG_VERSION'), message))
    else:
      print(message)
  else:
    print(level + message + ENDC)

def build(args):
  os_name = platform.system()
  use_shell = os_name == 'Windows'

  if not os.path.exists('deps'):
    os.makedirs('deps')
    log(MESSAGE, 'Downloading OBS dependencies')
    temp_path = os.path.join(tempfile.mkdtemp(), 'dependencies2017.zip')
    urlretrieve('https://obsproject.com/downloads/dependencies2017.zip', temp_path)

    log(MESSAGE, 'Extracting OBS dependencies')
    with zipfile.ZipFile(temp_path, 'r') as zf:
      zf.extractall('deps')

  if not os.path.exists('obs-studio'):
    log(MESSAGE, 'Cloning obs-studio')
    process = subprocess.Popen(['git', 'clone', '--recursive', 'https://github.com/obsproject/obs-studio'], shell=use_shell)
    process.wait()
    if process.returncode != 0:
      log(ERROR, 'Failed to clone obs studio')
      return 1

  build_type = "Debug" if args.debug else "Release"

  if not os.path.exists('obs-studio/build'):
    os.makedirs('obs-studio/build')

    log(MESSAGE, 'Running CMake')
    process = subprocess.Popen(['cmake', '..', '-G', 'Visual Studio 16 2019', '-A', 'x64', '-DCMAKE_BUILD_TYPE={}'.format(build_type), '-DDepsPath={}'.format(os.path.abspath('deps/win64')), '-DENABLE_UI=FALSE', '-DDISABLE_UI=TRUE', '-DENABLE_SCRIPTING=FALSE'], cwd='obs-studio/build', shell=use_shell)
    process.wait()
    if process.returncode != 0:
      log(ERROR, 'CMake failed')
      return 1

    log(MESSAGE, 'Running CMake build')
    process = subprocess.Popen(['cmake', '--build', '.', '--config', build_type], cwd='obs-studio/build', shell=use_shell)
    process.wait()
    if process.returncode != 0:
      log(ERROR, 'CMake build failed')
      return 1

  if not os.path.exists('cbuild'):
    os.makedirs('cbuild')

    log(MESSAGE, 'Running CMake')
    process = subprocess.Popen(['cmake', '..', '-G', 'Visual Studio 15 2017 Win64', '-DCMAKE_BUILD_TYPE={}'.format(build_type)], cwd='cbuild', shell=use_shell)
    process.wait()
    if process.returncode != 0:
      log(ERROR, 'CMake failed')
      return 1

    log(MESSAGE, 'Running CMake build')
    process = subprocess.Popen(['cmake', '--build', '.', '--config', build_type], cwd='cbuild', shell=use_shell)
    process.wait()
    if process.returncode != 0:
      log(ERROR, 'CMake build failed')
      return 1

  out_dir = os.path.abspath(os.path.join('build', build_type))
  if not os.path.exists(out_dir):
    shutil.copytree('obs-studio/build/rundir/{}'.format(build_type), out_dir)

  log(SUCCESS, 'Successfully built')

  return 0

def clean(args):
  if os.path.exists('obs-studio/build'):
    shutil.rmtree('obs-studio/build')

  if os.path.exists('build'):
    shutil.rmtree('build')

  if os.path.exists('cbuild'):
    shutil.rmtree('cbuild')

  if args.clean_src and os.path.exists('obs-studio'):
    shutil.rmtree('obs-studio')

  if args.clean_src and os.path.exists('deps'):
    shutil.rmtree('deps')

  return 0

def main():
  parser = argparse.ArgumentParser('build tool for scissors')
  parser.add_argument('action', choices=['build', 'clean'])
  parser.add_argument('--debug', action='store_true', default=os.environ.get('PROFILE') == 'debug', help="enables debug build")
  parser.add_argument('--clean-src', action='store_true', help="removes the source code directories for dependencies")

  args = parser.parse_args()

  if (args.action == 'build'):
    return build(args)
  elif (args.action == 'clean'):
    return clean(args)

if __name__ == '__main__':
  sys.exit(main())
