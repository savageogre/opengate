OpenGate
========

This is a free and open-source binaural beat generator, so that you can generate whatever binaural beat audio you
want, whether for meditative purposes, or however you want to use them.

See example yaml files in the ./beats directory.

The end goal is to generate a complete toolset for open-source consciousness expansion, where the audio files can be
generated dynamically by anyone, and shared freely under the GPLv3 license. These tools should be available to
_everyone_ and we should be able to create our own methodology which can be freely shared and freely improved by
anyone with the time and experience.

[Read about Binaural Beats on wikipedia.](https://simple.wikipedia.org/wiki/Binaural_beats)

The magic is in "entrainment", which is the process of nudging your brain's rhythms to follow the external beat, which
is produced by binaural beats. By playing slightly different tones in each ear, the brain perceives a third "beat".
Over time, the brain tends to align its dominant brainwave frequency with that beat.

This is called neural entrainment. Using the segments in the configuration files, you can generate any type of
binaural beat at any frequency, with transition segments which smoothly shift from one beat frequency to another,
helping the brain gradually follow along instead of being jolted.

See the file at `./beats/example_entrain.yaml` which generates a binaural beat like so:
 - 5 minutes at 7 Hz (theta)
 - 10 minutes in a linear transition from 7 Hz to 3.875 Hz (delta, for sleep or meditation)
 - finishes up with 15 minutes in 3.875 Hz (delta) so you can relax and meditate

It also customizes the theta carrier frequency at 200 Hz and delta at 100 Hz, so it sounds deeper and more relaxing.

Support Development
-------------------

I just created a Patreon, [here.](https://www.patreon.com/posts/opengate-free-139241387)

Feel free to support or not. The software will remain free and open-source.

Disclaimer
----------

This software and its author are not related to [the Monroe Institute](https://www.monroeinstitute.org/). They
authored the related Gateway Tapes, which has a reddit community here: [/r/gatewaytapes](https://www.reddit.com/r/gatewaytapes/)

This was made by [Savage Ogre](mailto:savageogre.music@gmail.com) with no affiliation to the Monroe Institute.

And if you want to hear [my unrelated psytrance on Sound Cloud...](https://soundcloud.com/ogresav)

Installation
------------

Install Rust and Cargo on your system if you don't have it (try `which cargo`):

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Finally:

    cargo build --release

You should see a `./target/release/opengate` binary after that.

**Note:** The makefile also allows `make && make install` if you have `~/.cargo/bin` in your `$PATH`.


Quickstart
----------

Install as mentioned above, then edit [./beats/meditation_delta.yaml](https://github.com/savageogre/opengate/blob/main/beats/meditation_delta.yaml) to your liking, and run:

    opengate your_new_beat.yaml -o my_binaural_beat.wav

Or, if you have recorded audio or text you would like to run Text-to-Speech against, try to work with the
[Liminal State meditation](https://github.com/savageogre/opengate/blob/main/openstates/001_liminal_state.yaml").

You will need to install piper for TTS, and you can find [all the releases here](https://github.com/rhasspy/piper/releases/tag/2023.11.14-2).
Install it, then ensure "piper" is in your $PATH variable, or pass `--piper-bin /path/to/piper` as an argument.

    opengate openstates/001_liminal_state.yaml -p /path/to/piper -o 001_liminal_state.wav


Using Text-To-Speech
--------------------

See the quickstart above.

[Install one of these piper releases.](https://github.com/rhasspy/piper/releases/tag/2023.11.14-2) and put `piper`
in your $PATH or pass it as `--piper-bin /path/to/piper`

See the [./openstates/001_liminal_state.yaml](https://github.com/savageogre/opengate/blob/main/openstates/001_liminal_state.yaml) example to see how to use TTS with opengate.

Installation With FLAC Support
------------------------------

FLAC support is optional since it relies on an external C library, and I figure you might just want to use something
like [Audacity](https://www.audacityteam.org/download/) to combine sounds or export in other formats, since those are
free.

**Note: If you want to edit and make your own audio, I would suggest saving them in a loss-less format for backup,
which would be WAV or FLAC primarily.**

On ubuntu/debian based systems, first install the dependencies, as it writes out WAV or FLAC files.
On fedora/red-hat that would be `flac-devel` or arch `flac`.

    sudo apt update
    sudo apt install libflac-dev

Now build with flac:

    cargo build --release --features flac

To use it, you literally just change the output file extension like so:

    opengate mybeat.yaml -o mybeat.flac

Usage
-----

Most of the options will be defined in the YAML config file that you use to generate your WAV or FLAC output file.

See examples in the `./beats` directory. Documentation on that YAML format is below.

But for basic purposes, you can edit one of those files or test with `./beats/test_short.yaml`, and you run opengate
like so:

    # WAV output (larger, uncompressed):
    opengate ./beats/test_short.yaml --out short.wav

    # flac output, if built-in (compressed but loss-less):
    opengate ./beats/test_short.yaml --out short.flac

It will process the YAML file, determine how best to render the file based on the wav or flac file extension, and
that's it!

Beat YAML Schema
----------------

Most of the magic is in the YAML config you use.

The general idea is that you enumerate a list of "segments", each is either of type "tone" or "transition".

A "tone" plays a specific carrier frequency in the left ear, and carrier frequency + desired hertz in the right ear.
Your brain will perceive the difference so that if the left ear hears 200 Hz, and the right ear 207 Hz, you will
perceive the 7 Hz "wobble" even if your headphones can't play 7 Hz at all.

We also play optional "noise", which can be of different colors: pink, white, brown.
Generally, most prefer pink or brown for meditation as they sound more calming, though you can experiment with any.

You will want to pick a "gain" for the tone, and a gain for the noise if you add it. Let's see an example segment:

    - type: tone
      dur: 60s
      gain: 0.25
      carrier: 200.0
      hz: 7.0
      noise:
        color: pink
        gain: 0.75

This Tone specification above will play 200 Hz in the left ear, 207 Hz in the right ear (entraining your brain to
7 hertz), and play pink noise 3 times louder than the tone. This will happen over 60 seconds (60 "dur" for duration).

*Duration Note*: Duration can take multiple formats! All of these work:

    1         # 1 second
    0.5       # half a second (requires leading zero)
    1s        # 1 second
    0.5s      # half a second
    15m       # 15 minutes
    1.5m      # a minute and a half, or...
    1m30s     # 1 minute and 30 seconds
    1h30m15s  # 1 hour, 30 minutes, and 15 seconds
    0.5h15s   # half an hour and 15 seconds
    0.1h0.1s  # a tenth of an hour and a tenth of a second (but why?)

*Gain Note*: You could technically put gain of each at 1.0, but it will normalize so that `gain_i = gain_i / total`,
so a `gain: 1.0` for the binaural tone and `gain: 1.0` for the noise` would be equivalent to 0.5 and 0.5 respectively.
Just as you see above the 0.25 and 0.75, that is equivalent to their ratio.

The other type of segment is a "transition".

This is very common for meditation purposes. Let's say you want to entrain your brain to 7 Hz theta and relax there
for 5 minutes, then gradually go down to 3.875 Hz delta over a period of 5 minutes, then meditate at a constant delta
for 20 minutes.

You will have to specify an initial "tone" segment at 7 Hz, and a final "tone" segment at 3.875 Hz, but the middle
segment should gradually curve from 7 Hz to 3.875 Hz. You might also want to control the gain and transition it as
well, make the noise sound louder, or even change from pink to brown noise (which will sound a bit abrupt, but that
should be fine).

This would be a potential transition:

    - type: transition
      dur: 5m
      curve: linear
      from:
        carrier: 200.0
        hz: 7.0
        gain: 0.25
        noise:
          color: pink
          gain: 0.75
      to:
        gain: 0.25
        carrier: 100.0
        hz: 3.875
        noise:
          color: pink
          gain: 0.75

You can see here we define a duration similarly (300 for 300 seconds or 5 minutes), and a curve (linear or exp),
then a `from` and `to` section which are very similar to what you'd specify for a tone, only they attribute to where
this segment starts and ends at.

Thus, our full meditation beat would be thus (also in beats/example.yaml):

    segments:
      - type: tone
        gain: 0.25
        dur: 5m
        carrier: 200.0
        hz: 7.0
        noise:
          color: pink
          gain: 0.75

      - type: transition
        dur: 5m
        curve: linear
        from:
          carrier: 200.0
          hz: 7.0
          gain: 0.25
          noise:
            color: pink
            gain: 0.75
        to:
          carrier: 100.0
          hz: 3.875
          gain: 0.25
          noise:
            color: pink
            gain: 0.75

      - type: tone
        dur: 20m
        carrier: 100.0
        hz: 3.875
        gain: 0.25
        noise:
          color: pink
          gain: 0.75

It's also possible to use [YAML anchors](https://medium.com/@kinghuang/docker-compose-anchors-aliases-extensions-a1e4105d70bd).

For a full and clean example of a beat using anchors and definitions, see ./beats/meditation_delta.yaml

Integrating Other Audio and Text-To-Speech
------------------------------------------

If you look at the quickstart above, you will see that it's possible to use text-to-speech.

First, specify two directories in the root of your schema like so:

    model_dir: "../text_to_speech/models"
    audio_dir: "../openstates_output/001_liminal_state"

Each is relative to the path of the actual yaml file. The `model_dir` points to the directory which has the .onnx
models you download. Every .onnx file will also need a related .onnx.json file.

The `audio_dir` is the relative directory where it will dump all the output audio from text-to-speech, or look for
WAV files you want to mix in. It will cache text-to-speech results there. If you want, you can pass `--force` or `-f`
to have it force generate all new text-to-speech.

The schema allows for mixed in text-to-speech and wav files through the `audio` key in each segment:

    - type: tone
      dur: 10m
      ...
      audio:
        - type: tts
          model: en_US-kristin-medium.onnx
          offset: 0s
          text: |
            This will play at the very beginning of this segment, given the zero s offset.
        - type: tts
          model: en_US-kristin-medium.onnx
          offset: 1m
          gain: 0.5  # defaults to 1.0
          text: "This will play at one minute."
        - type: file
          path: ./rain_sounds.wav  # relative to audio_dir
          gain: 0.75
          offset: 2m

Notice that there are two types of audio mixins, a `tts` or `file`. It _must_ be a wav file for now.
If you want them all to use the same model, use YAML anchors like in the file 001_liminal_state.yaml
