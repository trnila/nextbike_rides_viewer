<script>
  import L from "leaflet";
  import Datetime from "./Datetime.svelte";
  import Icon from '@iconify/svelte';


  let events = [];
  let current;
  let active = [];
  let playing = false;

  let M;

  const icon = L.divIcon({
    html: `<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" role="img" width="30" height="30" preserveAspectRatio="xMidYMid meet" viewBox="0 0 48 48"><path fill="currentColor" fill-rule="evenodd" d="m35.745 12.17l-4.925 1.48l3.28 8.578a8 8 0 1 1-1.868.715l-1.648-4.31l-5.682 11.802A1 1 0 0 1 24 31h-4.062A8.001 8.001 0 0 1 4 30a8 8 0 0 1 15.938-1h2.5l-4.88-13.664A.998.998 0 0 1 17.5 15H16a1 1 0 1 1 0-2h4.5a1 1 0 1 1 0 2h-.938l1.842 5.157l8.127-4.277l-.965-2.523a1 1 0 0 1 .647-1.315l5.957-1.787l.575 1.915Zm-13.662 9.89l1.972 5.52l4.23-8.784l-6.202 3.264Zm12.983 8.297l-2.113-5.527a6 6 0 1 0 1.868-.715l2.113 5.528a1 1 0 0 1-1.868.714ZM17.917 29H12a1 1 0 1 0 0 2h5.917A6.002 6.002 0 0 1 6 30a6 6 0 0 1 11.917-1Z" clip-rule="evenodd"/></svg>`,
    className: "",
    iconSize: [30, 30],
    iconAnchor: [15, 15],
  });

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
        let marker = L.marker([t.srcLat, t.srcLng], {icon: icon}).addTo(M);
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
