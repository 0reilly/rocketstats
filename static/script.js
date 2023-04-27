(function () {
    const scriptTag = document.querySelector('script[data-domain]');
    const dataDomain = scriptTag ? scriptTag.getAttribute('data-domain') : null;

    const eventData = {
        domain: dataDomain,
        url: window.location.href,
        referrer: document.referrer,
        device: {
            user_agent: navigator.userAgent,
        },
    };

    fetch('https://www.rocketstats.co/api/tracking/event', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(eventData),
    });
})();
