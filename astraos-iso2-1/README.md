# AstraOS ISO 2 Project

AstraOS ISO 2 is a project aimed at creating a bootable ISO image for the AstraOS operating system. This document provides an overview of the project structure, the purpose of each directory and file, and instructions for building the ISO.

## Project Structure

```
astraos-iso2
├── assets
│   ├── logos
│   └── wallpapers
├── iso
│   └── x86_64
│       ├── airootfs
│       │   └── etc
│       ├── packages.x86_64
│       └── profiledef.sh
├── scripts
│   ├── build-iso.sh
│   └── copy-assets.sh
├── .gitignore
└── README.md
```

### Directories and Files

- **assets/logos**: This directory will contain image files of the logos used in the ISO.
  
- **assets/wallpapers**: This directory will contain image files of the wallpapers used in the ISO.
  
- **iso/x86_64/airootfs**: This directory contains the root filesystem structure for the ISO. It includes a subdirectory `etc` for configuration files.
  
- **iso/x86_64/packages.x86_64**: This file lists the necessary packages for the ISO.
  
- **iso/x86_64/profiledef.sh**: This file is a profile definition script that configures the settings for the ISO.
  
- **scripts/build-iso.sh**: This script manages the ISO building process. It may include commands to assemble the necessary files and directories.
  
- **scripts/copy-assets.sh**: This script copies logos and wallpapers from the `assets` directory to the `airootfs` directory of the ISO. It also checks for file existence and displays log messages.
  
- **.gitignore**: This file specifies files and directories to be ignored by Git when tracking changes.

## Building the ISO

To build the ISO, run the `build-iso.sh` script located in the `scripts` directory. Ensure that all assets are in place and that the necessary packages are listed in `packages.x86_64`.

## Contribution

Contributions to the AstraOS ISO 2 project are welcome. Please follow the guidelines for submitting changes and ensure that your contributions adhere to the project's coding standards.

## License

This project is licensed under the terms of the MIT License. See the LICENSE file for details.