<template>
  <div>
    <div ref="zoomview" style="height: 200px;"></div>
    <div ref="overview" style="height: 80px;"></div>
    <audio ref="audio" :src="audioSrc" controls />
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'
import Peaks from 'peaks.js'

const audioSrc = '/path/to/your/audio.mp3'
const zoomview = ref(null)
const overview = ref(null)
const audio = ref(null)
let peaksInstance = null

onMounted(() => {
  const options = {
    zoomview: { container: zoomview.value },
    overview: { container: overview.value },
    mediaElement: audio.value,
    webAudio: { audioContext: new AudioContext() },
    emitCueEvents: true,
    showAxisLabels: true,
  }

  Peaks.init(options, (err, peaks) => {
    if (err) {
      console.error('Peaks.js init error:', err)
      return
    }
    peaksInstance = peaks
    // You can now use peaksInstance to add markers, regions, etc.
  })
})

onBeforeUnmount(() => {
  if (peaksInstance) {
    peaksInstance.destroy()
  }
})
</script>
