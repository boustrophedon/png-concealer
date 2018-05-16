# PNG Concealer
I was thinking about the chunk format of PNGs and remembered that there are text chunks, and for whatever reason decided that it would be interesting to see if it's possible to store data in a text chunk and have it survive popular image hosting websites.

This project contains a rust program that takes a source png file, a source data file, and outputs a new png that has the same chunks as the original plus an extra text chunk containing the source data.

This is similar to appending a zip file to another file, but has a higher likelihood to be preserved.

# How it works

PNG files (or "datastreams") are composed of a sequence of chunks, which contain a fixed-sized length field that gives the length of the data field, the type of the chunk, the data, and a crc checksum. 

In particular, there is a chunk type [for text](https://www.w3.org/TR/2003/REC-PNG-20031110/#11tEXt), which contains a data field with a null-terminated keyword followed by arbitrary text. Since the length field is fixed, we may only have 2^31-1 bytes, or approximately 2GB, of data.

Additionally, the last chunk in a PNG datastream is always the IEND chunk, which is simply the sequence `0000 0000 4945 4e44 ae42 6082`. The length is zero so we have four 0 bytes, then the bytes `49`, `45`, `4e`, `44` are the ascii values of IEND. The last four bytes are the CRC for the tag and data (there is no data) using the CRC polynomial defined in the PNG spec.

Therefore encoding concealed png is easy: take the original, remove the IEND, encode the tEXt chunk, and then write out the beginning of the original PNG, the tEXt chunk, and the IEND chunk.

Decoding: TODO

# This is not steganography

[Steganography](https://en.wikipedia.org/wiki/Steganography) is about hiding a message inside another message, and tries to provide plausible deniability to the fact that a message is being sent at all. The technique in this project is more like hiding a message in a false bottom of a suitcase: it's obvious that the message is there, but only if you look for it.

On the other hand, since we do not modify or even use the pixels of the image, this concealment doesn't show up if you were to compare the pixels of the image and the original (but will show up if you compare the image sizes).

# Further work

For smaller data files, it might be interesting to try and compress the original png image data such that the size of the output file is equal to the size of the input file. If the compressed png is less than `original - data`, then it may be possible to pad the remaining bytes with an extra ancillary block or split the IDAT blocks so that the overhead from the extra headers fills the gaps. Or simply padd the concealed data.

Additionally, the png format defines a compressed text chunk and an international text chunk (which can be further compressed with deflate). Either of those could be used to compress the source data file, though it makes more sense to simply compress the source data before base64 encoding in the first case. The international text chunk may allow us to use a more efficient encoding instead of base64 however (e.g. [base65536](https://github.com/qntm/base65536)).
