async function add_track_to_pl(trackId,playlistId) {
    const mk = MusicKit.getInstace();
    await mk.api.post('v1/me/library', {
        ids: {
            songs: [trackId]
        }
    });

    await mk.api.post(`v1/me/library/playlists/${playlistId}/tracks`, {
        data: [
            {
                id: trackId,
                type: "songs"
            }
        ]
    });
}