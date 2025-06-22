<template>
  <div>
    <div id="zoomview-container"></div>
    <div id="overview-container"></div>
    <audio ref="audio" controls>
      <source :src="audioSrc" type="audio/mpeg">
    </audio>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'
import Peaks from 'peaks.js'

// Replace with your actual audio file name stored in S3
const audioFileName = 'sound.wav'
const waveformFileName = 'sound'

// Audio and waveform URLs from your backend
const audioSrc = `http://localhost:8080/audio/${audioFileName}`
const waveformSrc = `http://localhost:8080/waveform/${waveformFileName}`

let peaksInstance = null

onMounted(async () => {
  try {
    // Fetch the waveform data from the backend
    const response = await fetch(waveformSrc)
    if (!response.ok) {
      throw new Error('Failed to fetch waveform data')
    }
    const waveformData = await response.json()

    // Initialize Peaks.js with dynamic data
    const options = {
      zoomview: {
        container: document.getElementById('zoomview-container')
      },
      overview: {
        container: document.getElementById('overview-container')
      },
      mediaElement: document.querySelector('audio'),
      dataUri: {
        arraybuffer: waveformData
      }
    }

    peaksInstance = await Peaks.init(options)

  } catch (err) {
    console.error('Error initializing Peaks:', err)
  }
})

onBeforeUnmount(() => {
  if (peaksInstance) {
    peaksInstance.destroy()
  }
})
</script>

