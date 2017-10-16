![Volunode](logo.svg.png)

Linux client for Berkeley Open Infrastructure for Network Computing.

[![](https://img.shields.io/badge/Chat-on%20Matrix-brightgreen.svg)](https://riot.im/app/#/room/#volunode:matrix.org)

## Running
### Dependencies
Volunode is developed against latest stable [Rust](https://rust-lang.org) toolchain including Cargo package manager.
Refer to your distribution's packaging or use [rustup](https://rustup.rs).

### Building and starting
```
$ cargo run
```

## Settings
Volunode is set up using a mix of environment variables and configuration files in the working directory.

### Environment variables
* `RPC_ADDR` - starts RPC server at address in the form of `IP:PORT`. RPC is disabled if this variable is not set.
* `RPC_PASSWORD` - enables RPC authentication and sets the password to provided string.

## License
Volunode is free software; you can redistribute it and/or modify it
under the terms of the GNU General Public License
as published by the Free Software Foundation,
either version 3 of the License, or (at your option) any later version.

Volunode is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with Volunode.  If not, see <http://www.gnu.org/licenses/>.
