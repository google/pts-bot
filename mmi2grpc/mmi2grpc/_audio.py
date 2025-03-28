# Copyright 2025 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import itertools
import math
import os
from threading import Thread

import numpy as np
from scipy.io import wavfile


def _fixup_wav_header(path):
    WAV_RIFF_SIZE_OFFSET = 4
    WAV_DATA_SIZE_OFFSET = 40

    with open(path, 'r+b') as f:
        f.seek(0, os.SEEK_END)
        file_size = f.tell()
        for offset in [WAV_RIFF_SIZE_OFFSET, WAV_DATA_SIZE_OFFSET]:
            size = file_size - offset - 4
            f.seek(offset)
            f.write(size.to_bytes(4, byteorder='little'))


SINE_FREQUENCY = 440
SINE_DURATION = 0.1

WAV_FILE = "/tmp/audiodata"


class AudioSignal:
    def __init__(self, transport, amplitude, fs):
        self.transport = transport
        self.amplitude = amplitude
        self.fs = fs
        self.thread = None

    def start(self):
        self.thread = Thread(target=self._run)
        self.thread.start()

    def _run(self):
        sine = self._generate_sine(SINE_FREQUENCY, SINE_DURATION)

        # Interleaved audio
        stereo = np.zeros(sine.size * 2, dtype=sine.dtype)
        stereo[0::2] = sine

        # Send 4 second of audio
        audio = itertools.repeat(stereo.tobytes(), int(4 / SINE_DURATION))

        self.transport(audio)

    def _generate_sine(self, f, duration):
        sine = self.amplitude * \
            np.sin(2 * np.pi * np.arange(self.fs * duration) * (f / self.fs))
        s16le = (sine * 32767).astype("<i2")
        return s16le

    def verify(self):
        assert self.thread is not None
        self.thread.join()
        self.thread = None

        _fixup_wav_header(WAV_FILE)

        samplerate, data = wavfile.read(WAV_FILE)
        # Take one second of audio after the first second
        audio = data[samplerate:samplerate*2, 0].astype(np.float) / 32767
        assert(len(audio) == samplerate)

        spectrum = np.abs(np.fft.fft(audio))
        frequency = np.fft.fftfreq(samplerate, d=1/samplerate)
        amplitudes = spectrum / (samplerate/2)
        index = np.where(frequency == SINE_FREQUENCY)
        amplitude = amplitudes[index][0]

        match_amplitude = math.isclose(
            amplitude, self.amplitude, rel_tol=1e-03)

        return match_amplitude
