<script>
  import L from "leaflet";

  let events = [];
  let current;
  let active = [];

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
      start();
    }

    //x();
    y();
  }

  let head = 0;

  function tick() {
    active = active.filter(a => {
      const t = events[a.index];
      if(current > t.tsDst) {
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

      let marker = L.marker([t.srcLat, t.srcLng]).addTo(M);
      var polyline = L.polyline([[t.srcLat, t.srcLng],[t.dstLat, t.dstLng]], {color: 'red'}).addTo(M);

      active.push({
        index: head,
        marker: marker,
        polyline: polyline,
      });
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

    window.requestAnimationFrame(move);
  }

  function start() {
    //tick();

    window.requestAnimationFrame(move);

    /*
    setInterval(() => {
      current = new Date(current.getTime() + 60*1000);
      tick();
    }, 1000);
    */
  }
</script>

<link
  rel="stylesheet"
  href="https://unpkg.com/leaflet@1.6.0/dist/leaflet.css"
  integrity="sha512-xwE/Az9zrjBIphAcBb3F6JVqxf46+CDLwfLMHloNu6KEQCAWi6HcDUbeOfBIptF7tcCzusKFjFw2yuvEpDL9wQ=="
  crossorigin=""
/>

{head}
{#if events.length}
  {events[0].tsSrc}

  {events[events.length - 1].tsSrc}
  <br>
{current}
{/if}

<button on:click={start}>Start</button>

<div style="width: 100vw; height: 100vh;" use:map />
