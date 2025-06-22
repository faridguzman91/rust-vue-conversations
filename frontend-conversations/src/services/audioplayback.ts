
const audioContext = new (window.AudioContext || window.webkitAudioContext)();

const analyser = audioContext.createAnalyser();
analyser.fftSize = 256; // Adjust this for better resolution

const sourceNode = audioContext.createMediaStreamSource(remoteStream);
sourceNode.connect(analyser);

const canvas = document.getElementById("waveformCanvas");
const canvasContext = canvas.getContext("2d");

function visualize() {
  requestAnimationFrame(visualize);
  
  const bufferLength = analyser.frequencyBinCount;
  const dataArray = new Uint8Array(bufferLength);
  
  analyser.getByteFrequencyData(dataArray);
  
  canvasContext.fillStyle = "rgb(200, 200, 200)";
  canvasContext.fillRect(0, 0, canvas.width, canvas.height);

  const barWidth = canvas.width / bufferLength;
  let barHeight;
  let x = 0;

  for (let i = 0; i < bufferLength; i++) {
    barHeight = dataArray[i];
    canvasContext.fillStyle = "rgb(" + (barHeight + 100) + ",50,50)";
    canvasContext.fillRect(x, canvas.height - barHeight / 2, barWidth, barHeight);
    x += barWidth + 1;
  }
}

visualize();
