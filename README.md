OpenGate
========

This is a free and open-source binaural beat generator, so that you can generate whatever binaural beat audio you
want - for meditative purposes, or however you want to use them. See example yaml files in the ./beats directory.

[Read about Binaural Beats on wikipedia.](https://simple.wikipedia.org/wiki/Binaural_beats)

This software and its author are not related to [the Monroe Institute](https://www.monroeinstitute.org/). They
authored the related Gateway Tapes, which has a reddit community here: [/r/gatewaytapes](https://www.reddit.com/r/gatewaytapes/)

[Savage Ogre](mailto:savageogre.music@gmail.com)

Installation
------------

On ubuntu/debian based systems, first install the dependencies, as it writes out WAV or FLAC files.
On fedora/red-hat that would be `flac-devel` or arch `flac`.

    sudo apt install libflac-dev

Install Rust and Cargo on your system if you don't have it (try `which cargo`):

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Finally:

    cargo build --release

You should see a `./target/release/opengate` binary after that.

Usage
-----

Most of the options will be defined in the YAML config file that you use to generate your WAV or FLAC output file.

See examples in the `./beats` directory. Documentation on that YAML format is below.

But for basic purposes, you can edit one of those files or test with `./beats/test_short.yaml`, and you run opengate
like so:

    # WAV output (larger, uncompressed):
    opengate ./beats/test_short.yaml --out short.wav

    # flac output (compressed but loss-less):
    opengate ./beats/test_short.yaml --out short.flac

It will process the YAML file, determine how best to render the file based on the wav or flac file extension, and
that's it!

Beat YAML Schema
----------------

Most of the magic is in the YAML config you use.

sample_rate: 48000
gain: 0.9
fade_ms: 50

segments:
  - type: tone
    dur: 3.0
    carrier: 200.0
    hz: 7.0

  - type: transition
    dur: 1.0
    from:
      carrier: 200.0
      hz: 7.0
    to:
      carrier: 100.0
      hz: 3.875
    # or "exp" 
    curve: linear

  - type: tone
    dur: 3.0
    carrier: 100.0
    hz: 3.875

