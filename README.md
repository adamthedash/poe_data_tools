# Parsing tools for Path of Exile Bundle files

## Dump Paths

## Dump files


# Bundle File format
![bundle file format](./images/bundle_spec.png)


**TODO List**
- Group files by their bundle before processing so we don't re-read the bundle a bunch of times
- Directly use Murmur64A as the Hasher for my LUTs, rather than using the hashes as keys with the default Hasher
- Proper error propogation in the lib crate using Anyhow
- Proper documentation for the lib crate
- Write up of bundle format to share with the dev community :)
