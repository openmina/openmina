// https://bugs.chromium.org/p/chromium/issues/detail?id=825576
// workaround: https://stackoverflow.com/questions/66546934/how-to-clear-closed-rtcpeerconnection-with-workaround
export function webrtcCleanup() {
  queueMicrotask(() => {
    console.warn("[WebRTC] doing heavy (around 50ms) GC for dangling peer connections");
    let img = document.createElement("img");
    img.src = window.URL.createObjectURL(new Blob([new ArrayBuffer(5e+7)])); // 50Mo or less or more depending as you wish to force/invoke GC cycle run
    img.onerror = function() {
      window.URL.revokeObjectURL(this.src);
      img = null
    }
  });
}

export function schedulePeriodicWebrtcCleanup() {
  setInterval(webrtcCleanup, 60 * 1000);
}
