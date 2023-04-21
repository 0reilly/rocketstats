(function() {
    function sendData() {
        var xhr = new XMLHttpRequest();

        var url = 'https://www.rocketstats.co/api/tracking/event';

        var data = {
            url: document.location.href,
            referrer: document.referrer,
            device: {
                userAgent: navigator.userAgent,
            },
        };

        xhr.open('POST', url, true);
        xhr.setRequestHeader('Content-Type', 'application/json;charset=UTF-8');
        xhr.send(JSON.stringify(data));
    }

    window.addEventListener('load', function() {
        sendData();
    });
})();
