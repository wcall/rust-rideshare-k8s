import random
import requests
import time
import threading
from datetime import datetime

HOSTS = [
    'us-east',
    'eu-north',
    'ap-south',
]

VEHICLES = [
    'bike',
    'scooter',
    'car',
]

def timestamp():
    return datetime.now().astimezone().isoformat(timespec='seconds')

def generate_load(host, vehicle):
    while True:
        start_time = time.time()
        print(f"{timestamp()} requesting {vehicle} from {host}")
        
        try:
            resp = requests.get(f'http://{host}:5000/{vehicle}')
            resp.raise_for_status()
            duration = time.time() - start_time
            print(f"{timestamp()} received {resp} in {duration:.2f}s from {host}/{vehicle}")
            
            # Sleep to complete the 4-second cycle
            sleep_time = max(4 - duration, 0)
            if sleep_time > 0:
                time.sleep(sleep_time)
                
        except BaseException as e:
            print(f"{timestamp()} http error for {host}/{vehicle}: {e}")
            # On error, still maintain the 10-second cycle
            time.sleep(4)

if __name__ == "__main__":
    print(f"{timestamp()} starting load generator with thread per region-vehicle combination")
    time.sleep(3)
    
    threads = []
    # Create one thread per host-vehicle combination
    for host in HOSTS:
        for vehicle in VEHICLES:
            thread = threading.Thread(target=generate_load, args=(host, vehicle))
            thread.daemon = True
            threads.append(thread)
            thread.start()
            print(f"{timestamp()} Started thread for {host}/{vehicle}")
    
    # Keep the main thread running
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print(f"{timestamp()} Shutting down load generator")
