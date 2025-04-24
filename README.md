<!--lint disable no-literal-urls-->
<div align="center">
  <h1>Aquarana</h1>
</div>
<div align="center">
  <strong>A pure rust implementation of the opus decoder.</strong>
</div>
<br>

---

This is a pure Rust implementation of the opus decoder, with no plans to support coding.

There have been some abandoned attempts at this type of project, partly due to the difficulty of implementing opus, which exists only in a vague RFC. The official libopus implementation is the only option at this point, but the libopus code is a very poor reading experience. It should be noted that there is an opus decoder implementation for ffmpeg, although the performance is not as good as libopus, the code is clear and it is a good reference project.

The current project does not start with ambitious goals that would make it difficult to implement, nor does it advocate surpassing other implementations, but rather, we are prepared to start with a simple goal of achieving a basically usable state.

Therefore, at this stage, we are not going to consider packet loss handling and FEC in real-time streaming scenarios, we are only going to support CELT decoders, and we are not going to think too much about performance optimization, working correctly is the main goal for now. For now, we will implement a decoder that only supports parsing CELT and mono/dual channels, which is the most basic implementation that can support decoding most opus-encoded audio files encapsulated in OGG.
