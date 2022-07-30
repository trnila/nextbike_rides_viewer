<script>
  import L from "leaflet";
  import Datetime from "./Datetime.svelte";
  import Icon from '@iconify/svelte';


  let events = [];
  let current;
  let active = [];
  let playing = false;

  let M;

  function map(container) {
    var map = L.map(container).setView([49.820923, 18.262524], 13);
    M = map;
    L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
      maxZoom: 19,
      attribution: "© OpenStreetMap",
    }).addTo(map);

    async function x() {
      const res = await fetch("stations.csv");
      const text = await res.text();
      text.split("\n").forEach((line) => {
        const [name, lat, lng] = line.split(", ");
        if (!lng) return;
        var marker = L.marker([lat, lng]).addTo(map);
        marker.bindPopup(name);
      });
    }

    async function y() {
      const res = await fetch("events.csv");
      const text = await res.text();

      events = text.split("\n").map((line) => {
        if (!line.length) return;
        let [id, tsSrc, src, tsDst, dst, srcLat, srcLng, dstLat, dstLng] =
          line.split(", ");

          tsSrc = new Date(tsSrc * 1000);
          tsDst = new Date(tsDst * 1000);

        return {
          id,
          tsSrc,
          src,
          tsDst,
          dst,
          srcLat,
          srcLng,
          dstLat,
          dstLng,
        };

      }).filter(el => el);

      current = events[0].tsSrc;

      console.log(events);
      start(true);
    }

    //x();
    y();
  }

  let head = 0;

  function tick() {
    active = active.filter(a => {
      const t = events[a.index];
      if(current > t.tsDst  || t.tsSrc > current) {
        a.marker.remove(M);
        a.polyline.remove(M);
        return false;
      }

      return true;
    });

    for(; head < events.length; head++) {
      const t = events[head];
      if(t.tsSrc > current) {
        break;
      }

      if(t.tsDst > current && !active.find((el) => el.index == head)) {
        let marker = L.marker([t.srcLat, t.srcLng]).addTo(M);
        var polyline = L.polyline([[t.srcLat, t.srcLng],[t.dstLat, t.dstLng]], {color: 'red'}).addTo(M);

        active.push({
          index: head,
          marker: marker,
          polyline: polyline,
        });
      }
    }
  }

  let last = 0;
  function move(timestamp) {
    if(last == 0) {
      last = timestamp;
      window.requestAnimationFrame(move);
      return;
    }

    const diff = timestamp - last;
    last = timestamp;

    current = new Date(current.getTime() + diff/1000 * 60*1000);
    tick();

    for(const a of active) {
      const b = events[a.index];
      const t = b.tsDst - current;
      
      let pos = a.marker.getLatLng();
      pos.lat += (b.dstLat - pos.lat) / t * 1000;
      pos.lng += (b.dstLng - pos.lng) / t * 1000;
      a.marker.setLatLng(pos);
    }

    if(playing) {
      window.requestAnimationFrame(move);
    }
  }

  function start(play) {
    playing = play;
    last = 0;
    head = 0;

    if(playing) {
      window.requestAnimationFrame(move);
    }
  }
</script>

<style>
  #toolbar {
    position: absolute;
    top: 0px;
    z-index: 1000;
    padding: 5px;
    width: 100vw;
    text-align: center;
  }

  button {
    cursor: pointer;
    width: 30px;
    height: 30px;
    line-height: 30px;
    background-color: #fff;
    border: 2px solid rgba(0,0,0,0.2);
    border-radius: 4px;
  }
  button:hover {
    background-color: #f4f4f4;
  }
</style>

<link
  rel="stylesheet"
  href="https://unpkg.com/leaflet@1.6.0/dist/leaflet.css"
  integrity="sha512-xwE/Az9zrjBIphAcBb3F6JVqxf46+CDLwfLMHloNu6KEQCAWi6HcDUbeOfBIptF7tcCzusKFjFw2yuvEpDL9wQ=="
  crossorigin=""
/>

<div id="toolbar">
  <button on:click={() => start(!playing)}>
    <Icon icon="el:{playing ? 'pause' : 'play'}" />
  </button>
  <Datetime bind:value={current} on:click={() => start(false)} />
</div>
<div style="width: 100%; height: 100%;" use:map />
