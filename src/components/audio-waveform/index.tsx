import { useEffect, useRef } from "react";

interface AudioWaveformProps {
  audioElement: HTMLAudioElement;
  width?: number;
  height?: number;
  barCount?: number;
  barColor?: string;
}

// 1. Use a SINGLE global AudioContext instance (Lazy initialized)
let globalAudioContext: AudioContext | null = null;

const getAudioContext = () => {
  if (!globalAudioContext) {
    globalAudioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
  }
  return globalAudioContext;
};

// Store source nodes to avoid "MediaElementAudioSourceNode can only be created once" errors
const audioSourceMap = new WeakMap<HTMLAudioElement, MediaElementAudioSourceNode>();

/**
 * Audio Waveform Visualization Component
 */
const AudioWaveform = ({
  audioElement,
  width = 56,
  height = 56,
  barCount = 40,
  barColor = "currentColor",
}: AudioWaveformProps) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationFrameRef = useRef<number | undefined>(undefined);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const timeoutRef = useRef<NodeJS.Timeout | undefined>(undefined);

  useEffect(() => {
    if (!audioElement || !canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const wasPlaying = !audioElement.paused;
    const currentTime = audioElement.currentTime;
    const volume = audioElement.volume;

    // 2. Retrieve the global singleton context
    const audioContext = getAudioContext();
    let source = audioSourceMap.get(audioElement);

    try {
      if (!source) {
        // Create source and connect to destination (speakers) only once per element
        source = audioContext.createMediaElementSource(audioElement);
        audioSourceMap.set(audioElement, source);
        source.connect(audioContext.destination);
      }

      // Create a new analyser for this visualization instance
      const analyser = audioContext.createAnalyser();
      analyser.fftSize = 256;
      analyser.smoothingTimeConstant = 0.8;

      // Connect source -> analyser
      source.connect(analyser);

      if (audioContext.state === "suspended") {
        audioContext.resume().catch(err => {
          console.warn("Failed to resume audio context:", err);
        });
      }

      // Restore audio state if messed up by connection
      audioElement.volume = volume;
      // Only set currentTime if significantly different to avoid glitches
      if (Math.abs(audioElement.currentTime - currentTime) > 0.1) {
        audioElement.currentTime = currentTime;
      }

      if (wasPlaying) {
        timeoutRef.current = setTimeout(() => {
          audioElement.play().catch(err => {
            console.warn("Failed to resume playback:", err);
          });
        }, 0);
      }

      analyserRef.current = analyser;
      canvas.width = width;
      canvas.height = height;

      const draw = () => {
        if (!analyser || !ctx) return;

        const bufferLength = analyser.frequencyBinCount;
        const dataArray = new Uint8Array(bufferLength);
        analyser.getByteFrequencyData(dataArray);

        ctx.clearRect(0, 0, width, height);

        const WAVEFORM_HEIGHT_SCALE_FACTOR = 0.8;
        const barWidth = width / barCount;
        const barGap = barWidth * 0.2;
        const actualBarWidth = barWidth - barGap;

        for (let i = 0; i < barCount; i++) {
          const dataIndex = Math.floor((i / barCount) * bufferLength);
          const barHeight = (dataArray[dataIndex] / 255) * height * WAVEFORM_HEIGHT_SCALE_FACTOR;

          const x = i * barWidth + barGap / 2;
          const y = height - barHeight;

          ctx.fillStyle = barColor;
          ctx.fillRect(x, y, actualBarWidth, barHeight);
        }

        animationFrameRef.current = requestAnimationFrame(draw);
      };

      draw();
    } catch (error) {
      console.error("Failed to create audio setup:", error);
    }

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
      // 3. Only disconnect the analyser. Do NOT close the context or disconnect the source from destination.
      if (analyserRef.current && source) {
        try {
          source.disconnect(analyserRef.current);
        } catch (error) {
          console.warn("Failed to disconnect analyser:", error);
        }
      }
      analyserRef.current = null;
    };
  }, [audioElement, width, height, barCount, barColor]);

  return <canvas ref={canvasRef} className="rounded-md" style={{ width, height }} />;
};

export default AudioWaveform;
