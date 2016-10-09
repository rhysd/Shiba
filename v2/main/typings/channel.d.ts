type ChannelFromMain
    = 'shiba:watch-error'
    | 'shiba:file-update'
    | 'shiba:dog-ready'
    | 'shiba:send-config'
;

type ChannelFromRenderer
    = 'shiba:tab-closed'
    | 'shiba:tab-opened'
    | 'shiba:request-config'
;
