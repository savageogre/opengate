OpenGate
========

This is a free and open-source binaural beat generator, so that you can generate whatever binaural beat audio you
want - for meditative purposes, or however you want to use them. See example yaml files in the ./beats directory.

[Read about Binaural Beats on wikipedia.](https://simple.wikipedia.org/wiki/Binaural_beats)

The magic is in "entrainment", which is the process of nudging your brain's rhythms to follow the external beat, which
is produced by binaural beats. By playing slightly different tones in each ear, the brain perceives a third "beat".
Over time, the brain tends to align its dominant brainwave frequency with that beat.

This is called neural entrainment. Using the segments in the configuration files, you can generate any type of
binaural beat at any frequency, with transition segments which smoothly shift from one beat frequency to another,
helping the brain gradually follow along instead of being jolted.

See the file at `./beats/entrain_theta_to_delta.yaml` which generates a binaural beat like so:
 - 5 minutes at 7 Hz (theta)
 - 10 minutes in a linear transition from 7 Hz to 3.875 Hz (delta, for sleep or meditation)
 - finishes up with 15 minutes in 3.875 Hz (delta) so you can relax and meditate

It also customizes the theta carrier frequency at 200 Hz and delta at 100 Hz, so it sounds deeper and more relaxing.

Disclaimer
----------

This software and its author are not related to [the Monroe Institute](https://www.monroeinstitute.org/). They
authored the related Gateway Tapes, which has a reddit community here: [/r/gatewaytapes](https://www.reddit.com/r/gatewaytapes/)

This was made by [Savage Ogre](mailto:savageogre.music@gmail.com) with no affiliation to the Monroe Institute.

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
Let's look at beats/test_short.yaml as an example:

    sample_rate: 48000

    # Overall volume multiplier.
    # 0.0 = silent, 1.0 = full scale. 
    # 0.9 means "90% of max". This keeps the audio strong while leaving headroom to avoid clipping.
    gain: 0.9

    # Short fade-in/out in milliseconds, applied at tone starts/ends.
    # Prevents audible clicks when waveforms change abruptly.
    fade_ms: 50

    # This is a list of segments of the song.
    segments:
      # A tone segment is a fixed-frequency carrier with binaural beat offset.
      # It persists this binaural beat for a duration with no changes in phase.
      - type: tone
        # Duration in seconds.
        dur: 3.0
        # Base frequency sent to the left ear.
        carrier: 200.0
        # Beat frequency (it's the difference between the left and right ears).
        # Left ear = carrier frequency (200 Hz)
        # Right ear = carrier + hz (207 Hz)
        # This produces a 7 Hz binaural beat, which is in the theta range.
        hz: 7.0

      # A gradual glide from one tone to another.
      # This allows "entrainment", the process of nudging the brain’s rhythms to follow an external beat.
      # A transition segment smoothly shifts from one beat frequency to another,
      # helping the brain gradually follow along instead of being jolted to a new frequency.
      # For example, moving from ~7 Hz (theta, relaxed focus) to 3.875 Hz (deep theta or delta) supports easing into a
      # meditative or hypnagogic state.
      - type: transition

        # Duration of the transition in seconds.
        dur: 1.0

        from:
          # Starting carrier frequency
          carrier: 200.0
          # Starting binaural beat frequency
          hz: 7.0

        to:
          # Ending carrier frequency
          carrier: 100.0
          # Ending binaural beat frequency
          hz: 3.875

        # This chooses how the transition interpolates.
        # You have two options:
        #   - linear: straight slope
        #   - exp: exponential curve
        curve: linear

      # Another final fixed tone segment after the transition, keeping at 3.875.
      - type: tone
        # Duration in seconds
        dur: 3.0
        # Lower carrier frequency for a more relaxed tone
        carrier: 100.0
        # Beat frequency in the delta range
        hz: 3.875
