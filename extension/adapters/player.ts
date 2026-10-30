/** Browser media interoperability. Rust owns the markup and public source URLs. */
const boundPlayers = new WeakSet<HTMLElement>();
function enhanceMedia(root: ParentNode): void {
  for (const player of root.querySelectorAll<HTMLElement>('.lx-player')) {
    if (boundPlayers.has(player)) continue;
    const video = player.querySelector('video');
    if (!video) continue;
    boundPlayers.add(player);
    const command = (name: string) => player.querySelector<HTMLButtonElement>(`[data-media-command="${name}"]`);
    const seek = player.querySelector<HTMLInputElement>('[data-media-role="seek"]');
    const volume = player.querySelector<HTMLInputElement>('[data-media-role="volume"]');
    const speed = player.querySelector<HTMLSelectElement>('[data-media-role="speed"]');
    const status = player.querySelector<HTMLElement>('[data-media-role="status"]');
    const source = player.querySelector<HTMLAnchorElement>('.lx-media-source');
    const time = player.querySelector<HTMLElement>('[data-media-role="time"]');
    const clock = (value: number) => {
      const seconds = Number.isFinite(value) ? Math.max(0, Math.floor(value)) : 0;
      return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
    };
    const update = () => {
      player.dataset.playing = String(!video.paused && !video.ended);
      command('play')?.setAttribute('aria-label', video.paused ? 'Play video' : 'Pause video');
      command('mute')?.setAttribute('aria-label', video.muted ? 'Unmute video' : 'Mute video');
      command('mute')?.setAttribute('aria-pressed', String(video.muted));
      command('fullscreen')?.setAttribute('aria-label', player.matches(':fullscreen') ? 'Exit full screen' : 'Full screen');
      if (seek) {
        seek.value = String(Number.isFinite(video.duration) && video.duration > 0 ? video.currentTime / video.duration * 1000 : 0);
        seek.disabled = !Number.isFinite(video.duration) || video.duration <= 0;
        seek.setAttribute('aria-valuetext', `${clock(video.currentTime)} of ${clock(video.duration)}`);
      }
      if (volume) volume.value = String(video.muted ? 0 : video.volume);
      if (time) time.textContent = `${clock(video.currentTime)} / ${clock(video.duration)}`;
    };
    const play = async () => {
      try { await video.play(); if (status) status.hidden = true; if (source) source.hidden = true; }
      catch { if (source) source.hidden = false; if (status) { status.hidden = false; status.textContent = 'Video could not start. Try again or open it on X.'; } }
      update();
    };
    player.addEventListener('click', event => {
      const button = event.target instanceof Element ? event.target.closest<HTMLButtonElement>('[data-media-command]') : null;
      if (!button || !event.isTrusted) return;
      event.stopPropagation();
      switch (button.dataset.mediaCommand) {
        case 'play': if (video.paused) void play(); else video.pause(); break;
        case 'mute': video.muted = !video.muted; break;
        case 'retry': video.load(); void play(); break;
        case 'fullscreen':
          if (player.matches(':fullscreen')) void document.exitFullscreen();
          else void player.requestFullscreen().catch(() => { if (status) { status.hidden = false; status.textContent = 'Full screen is unavailable in this browser.'; } });
          break;
      }
      update();
    });
    seek?.addEventListener('input', () => { if (Number.isFinite(video.duration)) video.currentTime = Number(seek.value) / 1000 * video.duration; update(); });
    volume?.addEventListener('input', () => { video.volume = Number(volume.value); video.muted = video.volume === 0; update(); });
    speed?.addEventListener('change', () => { video.playbackRate = Number(speed.value); });
    for (const name of ['play','pause','ended','timeupdate','loadedmetadata','volumechange','durationchange']) video.addEventListener(name, update);
    player.addEventListener('fullscreenchange', update);
    video.addEventListener('error', () => {
      if (source) source.hidden = false;
      if (status) { status.hidden = false; status.textContent = 'Video could not load. Retry or open it on X.'; }
      const retry = command('retry'); if (retry) retry.hidden = false;
    });
    const retry = command('retry'); if (retry) retry.hidden = true;
    if (status) status.hidden = true;
    if (source) source.hidden = true;
    if (speed) speed.value = String(video.playbackRate);
    video.controls = false;
    player.dataset.enhanced = 'true';
    update();
  }
}
function pauseMedia(root: ParentNode): void { for (const video of root.querySelectorAll('video')) video.pause(); }
