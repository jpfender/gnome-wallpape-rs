# GNOME-Wallpape-rs

A simple random wallpaper changer for GNOME written in Rust.
Can change the wallpaper as a one-off or run indefinitely,
changing wallpapers in a given interval.
Allows specifying multiple wallpaper directories and cycling between them on demand.
Can be reconfigured at runtime.

## Usage

Create a TOML config file
(default path file is `~/wallch.toml`)
with at least the following contents:

```toml
dirs = ["<Your wallpaper directory>"]
```

You can also specify multiple wallpaper directories as well as a duration between cycles:

```toml
dirs = ["<First directory>", "<Second directory>"]
duration = "10m"
```

The first entry in `dirs` is selected as the default,
but you can cycle through the list by calling:

```
$ wallch toggle
```

This will set the current directory persistently by updating the config file with the `current` parameter
(which is the index of `dirs` and can also be set manually).

To run indefinitely, call:

```
$ wallch run
```

The default interval between wallpaper changes is 10 minutes if no duration is specified.
This can be changed at runtime by editing the config or calling:

```
$ wallch [-d/--duration] <LENGTH>
```

However,
the new duration will only be applied after the current cycle is complete.

To switch to the next wallpaper in the current directory, call:

```
$ wallch next
```

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/jpfender/gnome-wallpape-rs/tags).

## Authors

- **Jakob Pfender** - _Initial work_ - [jpfender](https://github.com/jpfender)

<!--See also the list of [contributors](https://github.com/jpfender/gnome-wallpape-rs/contributors) who participated in this project.-->

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details
