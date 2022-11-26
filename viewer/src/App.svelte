<script>
  import L from "leaflet";
  import Datetime from "./Datetime.svelte";
  import Icon from "@iconify/svelte";

  function getStartTime() {
    let time = new Date();
    //time.setHours(time.getHours() - 24);
    time.setDate(time.getDate() - 10);
    time.setHours(10);
    return time;
  }

  let events = [];
  let current = getStartTime();
  let active = [];
  let playing = false;
  let sim_time_per_s = 60;
  let stations;
  let last_event_id = -1;

  let M;
  let promise = null;

  const icon = L.divIcon({
    html: `<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" role="img" width="30" height="30" preserveAspectRatio="xMidYMid meet" viewBox="0 0 48 48"><path fill="currentColor" fill-rule="evenodd" d="m35.745 12.17l-4.925 1.48l3.28 8.578a8 8 0 1 1-1.868.715l-1.648-4.31l-5.682 11.802A1 1 0 0 1 24 31h-4.062A8.001 8.001 0 0 1 4 30a8 8 0 0 1 15.938-1h2.5l-4.88-13.664A.998.998 0 0 1 17.5 15H16a1 1 0 1 1 0-2h4.5a1 1 0 1 1 0 2h-.938l1.842 5.157l8.127-4.277l-.965-2.523a1 1 0 0 1 .647-1.315l5.957-1.787l.575 1.915Zm-13.662 9.89l1.972 5.52l4.23-8.784l-6.202 3.264Zm12.983 8.297l-2.113-5.527a6 6 0 1 0 1.868-.715l2.113 5.528a1 1 0 0 1-1.868.714ZM17.917 29H12a1 1 0 1 0 0 2h5.917A6.002 6.002 0 0 1 6 30a6 6 0 0 1 11.917-1Z" clip-rule="evenodd"/></svg>`,
    className: "",
    iconSize: [30, 30],
    iconAnchor: [15, 15],
  });

  async function y() {
    if (!stations) {
      const res = await fetch("stations.json");
      stations = await res.json();
    }

    function getUrl() {
      if (last_event_id >= 0) {
        return `rides.json?last_event_id=${last_event_id}`;
      }
      return `rides.json?from=${parseInt(current / 1000)}`;
    }

    const res = await fetch(getUrl());
    const json = await res.json();

    events = [
      ...events,
      ...json.rides
        .map((ride) => {
          try {
            ride.src.timestamp = new Date(ride.src.timestamp * 1000);
            ride.dst.timestamp = new Date(ride.dst.timestamp * 1000);
            ride.src.lat = stations[ride.src.station_id].lat;
            ride.src.lng = stations[ride.src.station_id].lng;
            ride.dst.lat = stations[ride.dst.station_id].lat;
            ride.dst.lng = stations[ride.dst.station_id].lng;
            return ride;
          } catch (err) {
            console.log(err);
            return null;
          }
        })
        .filter((record) => record),
    ];

    last_event_id = json.last_event_id;
    start(true);
  }

  function map(container) {
    var map = L.map(container).setView([49.820923, 18.262524], 13);
    M = map;
    L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
      maxZoom: 19,
      attribution: "© OpenStreetMap",
    }).addTo(map);

    async function x() {
      const res = await fetch("stations.json");
      const json = await res.json();
      for (const [_, station] of Object.entries(json)) {
        let marker = L.marker([station.lat, station.lng]).addTo(map);
        marker.bindPopup(station.name);
      }
    }

    //x();
    y();
  }

  let head = 0;

  function tick() {
    active = active.filter((a) => {
      const t = events[a.index];
      if (current > t.dst.timestamp || t.src.timestamp > current) {
        a.marker.remove(M);
        a.polyline.remove(M);
        return false;
      }

      return true;
    });

    for (; head < events.length; head++) {
      const t = events[head];
      if (t.src.timestamp > current) {
        break;
      }

      if (t.dst.timestamp > current && !active.find((el) => el.index == head)) {
        const pad = (s) => ("00" + s).substr(-2);
        const time = (t) => `${pad(t.getHours())}:${pad(t.getMinutes())}`;

        const diff = parseInt(
          (t.dst.timestamp - t.src.timestamp) / 1000 / sim_time_per_s
        );

        let marker = L.marker([t.src.lat, t.src.lng], { icon: icon }).addTo(M);
        marker.bindPopup(
          `<strong>${t.bike_id}</strong><br>${time(t.src.timestamp)} - ${time(
            t.dst.timestamp
          )}<br>${diff} min`
        );
        var polyline = L.polyline(
          [
            [t.src.lat, t.src.lng],
            [t.dst.lat, t.dst.lng],
          ],
          { color: "red" }
        ).addTo(M);

        active.push({
          index: head,
          marker: marker,
          polyline: polyline,
        });
      }
    }

    if (events.length - head < 85 && !promise) {
      promise = y();
      promise.then(() => (promise = null));
      promise.catch(() => (promise = null));
    }
  }

  let last = 0;
  function move(timestamp) {
    if (last == 0) {
      last = timestamp;
      window.requestAnimationFrame(move);
      return;
    }

    const diff = timestamp - last;
    last = timestamp;

    current = new Date(
      current.getTime() + (diff / 1000) * sim_time_per_s * 1000
    );
    tick();

    for (const a of active) {
      const b = events[a.index];
      const t = b.dst.timestamp - current;

      let pos = a.marker.getLatLng();
      pos.lat += ((b.dst.lat - pos.lat) / t) * 1000;
      pos.lng += ((b.dst.lng - pos.lng) / t) * 1000;
      a.marker.setLatLng(pos);
    }

    if (playing) {
      window.requestAnimationFrame(move);
    }
  }

  function start(play) {
    playing = play;
    last = 0;
    head = 0;

    if (playing) {
      window.requestAnimationFrame(move);
    }
  }
</script>

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
  <input type="number" bind:value={sim_time_per_s} />
</div>
<div style="width: 100%; height: 100%;" use:map />

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
    border: 2px solid rgba(0, 0, 0, 0.2);
    border-radius: 4px;
  }
  button:hover {
    background-color: #f4f4f4;
  }
</style>
