import { Injectable } from '@angular/core';
import { RustService } from '@core/services/rust.service';
import { map, Observable, of } from 'rxjs';
import { NetworkNodeDHT } from '@shared/types/network/node-dht/network-node-dht.type';

@Injectable({
  providedIn: 'root'
})
export class NetworkNodeDhtService {

  constructor(private rust: RustService) {
  }

  getDhtPeers(): Observable<{ peers: NetworkNodeDHT[], thisKey: string }> {
    // return this.rust.get<DhtPeersResponse>('/discovery/routing_table').pipe(
    return of({
      'this_key': '161103ba59457878927b8fa18bb41618f0c6d5e50bd8955ed6ecc6fdce1fee5d', 'buckets': [{
        'max_dist': 'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff',
        'entries': [{
          'peer_id': '2bbTgVnscCDkFM52oJsFvokJLvjdgsWGJCj2urfyEJ5dcab4GAR',
          'libp2p': '12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag',
          'key': 'cc104522316161701d4d19fbe0cf4e7b897d6eb5b7e5ba51150132deeba35dc6',
          'dist': 'da014698682419088f36965a6b7b586379bbbb50bc3d2f0fc3edf42325bcb39b',
          'addrs': ['/ip4/34.135.63.47/tcp/10001']
        }, {
          'peer_id': '2av23g9QVh64Cy2FEEE4Y6geYxFaD8bfxNgUSTCtbCrkPqx6pSo',
          'libp2p': '12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv',
          'key': '904ea84fac9c6d8ca76a0ca6f4e71bc8c6ed4566fd0986dbac199871bc43bc4a',
          'dist': '865fabf5f5d915f4351183077f530dd0362b9083f6d113857af55e8c725c5217',
          'addrs': ['/ip4/34.170.114.52/tcp/10002', '/ip4/127.0.0.1/tcp/10001', '/ip4/10.4.5.2/tcp/10001', '/ip4/34.136.110.79/tcp/10001', '/ip4/34.136.110.79/tcp/29167']
        }, {
          'peer_id': '2afgQgs7674WWiwaPEQniA5aJgPgcF4EMd1ge82Yy1tWPMNjVhD',
          'libp2p': '12D3KooWCXBDYKmP7ZKWrcoNxTgscJpuysBH9dvo79vnEzYnE4hJ',
          'key': '8d8bf97fde48d97fa5e8bcab82572fc1676e7933640c6b6aa556ea88e8d8f479',
          'dist': '9b9afac5870da1073793330a09e339d997a8acd66fd4fe3473ba2c7526c71a24',
          'addrs': ['/ip4/127.0.0.1/tcp/10502', '/ip4/34.122.106.164/tcp/10502', '/ip4/34.122.106.164/tcp/59129', '/ip4/10.4.52.6/tcp/10502']
        }, {
          'peer_id': '2aS8KzfRsATyexah4KQvT6KsQ8JChdcRipTo2Xhp7QCpXWLMknn',
          'libp2p': '12D3KooWAT3k3zY7cAZmTfX7hXkpQsGYbXimyPpinCc7Cy4z1D18',
          'key': 'f8ca3e5f15fb14059bae8608b4477181273264b47da9a986048b518b0524831b',
          'dist': 'eedb3de54cbe6c7d09d509a93ff36799d7f4b15176713cd8d2679776cb3b6d46',
          'addrs': ['/ip4/127.0.0.1/tcp/10515', '/ip4/34.27.206.250/tcp/10515', '/ip4/34.27.206.250/tcp/7041', '/ip4/10.4.69.9/tcp/10515']
        }, {
          'peer_id': '2asUkYGod8U4K5KXULBx1NY1KZo1nVixE67hr3RHzPxKi2KjXbS',
          'libp2p': '12D3KooWEKkvaZqCRft22GYnB4kvYfyrcFXseWwpvKjekHykxorN',
          'key': '8e0a771ae4a6a2045fe0780c8bb63c670cbd66fe7500a26f89006b100ca342e7',
          'dist': '981b74a0bde3da7ccd9bf7ad00022a7ffc7bb31b7ed837315fecadedc2bcacba',
          'addrs': ['/ip4/127.0.0.1/tcp/10501', '/ip4/35.225.195.143/tcp/10501', '/ip4/35.225.195.143/tcp/52534', '/ip4/10.4.10.10/tcp/10501']
        }, {
          'peer_id': '2aRb8Jo5o3owMuZ3WK3bo12wdpW5e9RhEdw5MCmnZDBZuTEL9i4',
          'libp2p': '12D3KooWANHBnNJ5ia7gxn8fXiNNieV9KWqxkjdBP4s1BUByibzM',
          'key': 'a8f2b2e3b3fb25429b6384e76959f4903fe904587a77b331fafa69b1372211a4',
          'dist': 'bee3b159eabe5d3a09180b46e2ede288cf2fd1bd71af266f2c16af4cf93dfff9',
          'addrs': ['/ip4/127.0.0.1/tcp/10509', '/ip4/35.192.35.47/tcp/10509', '/ip4/35.192.35.47/tcp/54722', '/ip4/10.6.35.6/tcp/10509']
        }, {
          'peer_id': '2ajD7CfVgK4FvhmiSAprnguwmDs3HnGL9mU7fAJxENZW6yE4GF1',
          'libp2p': '12D3KooWD4TaFu8qFtBzDB2n8YRYeDAJx8hoDZPzAwbPH8hLHz4Z',
          'key': '8caf4dc16d522b6d026472dac1ad0223bb507a53fb20cbdab624d85f8cb97db6',
          'dist': '9abe4e7b34175315901ffd7b4a19143b4b96afb6f0f85e8460c81ea242a693eb',
          'addrs': ['/ip4/127.0.0.1/tcp/10512', '/ip4/34.135.28.97/tcp/10512', '/ip4/34.135.28.97/tcp/10220', '/ip4/10.4.122.18/tcp/10512']
        }, {
          'peer_id': '2aXmXohNQkfkLAdXS1GzHomVNMzAph5ZjoWyEREe1VWD9WD6vTo',
          'libp2p': '12D3KooWBK3vz1inMubXCUeDF4Min6eG5418toceG8QvNPWRW1Gz',
          'key': 'd33c28a610508febc0b27c2c79295e09a1caa1ce33679e1e6c161ec69cb49dab',
          'dist': 'c52d2b1c4915f79352c9f38df29d4811510c742b38bf0b40bafad83b52ab73f6',
          'addrs': ['/ip4/127.0.0.1/tcp/8302', '/ip4/65.109.123.235/tcp/8302']
        }, {
          'peer_id': '2cETyFw4D9hKYSG39ZcUcfZode6jKeLezJteJKzsnxHRMR2Y6V2',
          'libp2p': '12D3KooWSPrpzVgYzP9F9DbqeZfcCVNhShXMEAxuSCBSq4vqHkNd',
          'key': 'd892bf06f911a0400717ab0e4fb7e04a1f41c95c2d19a1e693ea6bcc2d87887c',
          'dist': 'ce83bcbca054d838956c24afc403f652ef871cb926c134b84506ad31e3986621',
          'addrs': ['/ip4/127.0.0.1/tcp/10909', '/ip4/34.123.25.225/tcp/10909', '/ip4/34.123.25.225/tcp/53135', '/ip4/10.4.209.5/tcp/10909']
        }, {
          'peer_id': '2akvMijZ4oVdSmyL2H3KmXHjppAybVWwjR9qSWSbDcmWaXNCwFU',
          'libp2p': '12D3KooWDKdHVbnkM7GJYML6ogYR5KmHUj9Ngnq1Lk42xcXnf2sx',
          'key': 'fdd06af8c824b07701eb86d41b2ea84f070ae9b978565e1025be21255bba468f',
          'dist': 'ebc169429161c80f93900975909abe57f7cc3c5c738ecb4ef352e7d895a5a8d2',
          'addrs': ['/ip4/127.0.0.1/tcp/8302', '/ip4/172.18.0.15/tcp/8302', '/ip4/176.9.123.23/tcp/8302']
        }, {
          'peer_id': '2aqwdRqmuuuWNMXaL8FXVrwGn9fNqBAKc6BisVB56U5BDN8brLL',
          'libp2p': '12D3KooWE68ys66LSbX8LhNK2vJdxYks3YjQf3R7vXAX3yaNLKbw',
          'key': 'f4598f4e2ba6fe46cb591778c1b7895d212ae0be9f166347ea1c73fa8316745b',
          'dist': 'e2488cf472e3863e592298d94a039f45d1ec355b94cef6193cf0b5074d099a06',
          'addrs': ['/ip4/172.18.0.3/tcp/8302', '/ip4/127.0.0.1/tcp/8302', '/ip4/142.132.154.120/tcp/8302', '/ip4/142.132.154.120/tcp/51006', '/ip4/142.132.154.120/tcp/4622']
        }, {
          'peer_id': '2aT9mcnstnc2A3Eqsyjn9SKZdwQxXg5CE734VSkGQfs45XkSPHG',
          'libp2p': '12D3KooWAc8cFxFzPnnw2z1pYqs2vFHG5ATMm44zzKXqoogtEYuZ',
          'key': 'd4f46416cb3c390f6a679d122d834151db6626954a8cafb61f9cc1ef6d8fb045',
          'dist': 'c2e567ac92794177f81c12b3a63757492ba0f37041543ae8c9700712a3905e18',
          'addrs': ['/ip4/127.0.0.1/tcp/10511', '/ip4/35.224.101.158/tcp/10511', '/ip4/35.224.101.158/tcp/40998', '/ip4/10.5.213.17/tcp/10511']
        }, {
          'peer_id': '2bdqNQXxthS5C6Khv6jBcW92djBQudxiotG4Hs4XPdW7f945eS9',
          'libp2p': '12D3KooWM6uUe9Y7mZ56zGCZMCPXD5VHqSE8vCB35ifxVCReXVzn',
          'key': '8e60082a827b325dc9740dc8680d6c4c3eec8c439e2413545505990db19b5871',
          'dist': '98710b90db3e4a255b0f8269e3b97a54ce2a59a695fc860a83e95ff07f84b62c',
          'addrs': ['/ip4/127.0.0.1/tcp/10909', '/ip4/34.122.106.164/tcp/10909', '/ip4/34.122.106.164/tcp/1320', '/ip4/10.4.52.5/tcp/10909']
        }, {
          'peer_id': '2aobvaEx6GWRxy9D6okoXXxEcpMB8Le3P65nQaimJ4LWZm1RcaR',
          'libp2p': '12D3KooWDjQA5gVz96Q14LuLpNY4ELHUHigVsThggh3qXEaJWhBd',
          'key': 'f7a0438d7004ec5cf09e3ab87acae823ada258bfcc78f41e6cd583bab573b7e2',
          'dist': 'e1b140372941942462e5b519f17efe3b5d648d5ac7a06140ba3945477b6c59bf',
          'addrs': ['/ip4/35.224.101.158/tcp/21189', '/ip4/127.0.0.1/tcp/10504', '/ip4/35.224.101.158/tcp/10504', '/ip4/10.5.213.16/tcp/10504']
        }]
      }, {
        'max_dist': '7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff',
        'entries': [{
          'peer_id': '2bm5SG9cjzr4TYX7SofprnhZuji8F4khoycgt5wQ54sgjieJs8b',
          'libp2p': '12D3KooWND6m6GBtwzhCsEZA61pkLMGHvXHgbv1H6SfkbCQT7uPt',
          'key': '161103ba59457878927b8fa18bb41618f0c6d5e50bd8955ed6ecc6fdce1fee5d',
          'dist': '0000000000000000000000000000000000000000000000000000000000000000',
          'addrs': ['/ip4/127.0.0.1/tcp/8302']
        }, {
          'peer_id': '2aTKv6vapFoRUoKea9FTD3vxJm5qiYBMJcD2mGyjgdcryXGHHFt',
          'libp2p': '12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs',
          'key': '09e748eee6d5eab12eb5a5e13dc9223992274e38c32517b18d4d0a98c4c6a0a5',
          'dist': '1ff64b54bf9092c9bcce2a40b67d342162e19bddc8fd82ef5ba1cc650ad94ef8',
          'addrs': ['/ip4/34.170.114.52/tcp/10000', '/ip4/127.0.0.1/tcp/10001', '/ip4/10.4.3.2/tcp/10001', '/ip4/104.197.239.248/tcp/10001', '/ip4/104.197.239.248/tcp/1225', '/ip4/34.70.183.166/tcp/10001']
        }, {
          'peer_id': '2c5FNkMzdekEqSDyyQJ5BmVLeHaons3JiknX8cewgDvG1mBLEqU',
          'libp2p': '12D3KooWQzAQzFzLirCZG1aAFdahkFEUWwujuYAdJeuL9n2nWDyS',
          'key': '3aa080da598c47a1bd15626358fa7911a6d4481640acd16dcb8e990fab2a6fbb',
          'dist': '2cb1836000c93fd92f6eedc2d34e6f0956129df34b7444331d625ff2653581e6',
          'addrs': ['/ip4/127.0.0.1/tcp/10510', '/ip4/35.192.35.47/tcp/10510', '/ip4/35.192.35.47/tcp/33948', '/ip4/10.6.35.5/tcp/10510']
        }, {
          'peer_id': '2av5fzwMGprmF69c3syE5RFy75Ky7749J8rhKLUg1W97RddBLgw',
          'libp2p': '12D3KooWEipdNx2PzWer1QcKGL3gdqLARaAFwLf3YJkZEQSEdWmW',
          'key': '401386590591d33f9db4a1de9da9dea55091492f9a0f8fb9c110559286a1d6cc',
          'dist': '560285e35cd4ab470fcf2e7f161dc8bda0579cca91d71ae717fc936f48be3891',
          'addrs': ['/ip4/127.0.0.1/tcp/10501', '/ip4/34.171.103.38/tcp/10501', '/ip4/34.171.103.38/tcp/21349', '/ip4/34.171.103.38/tcp/54467', '/ip4/10.4.146.13/tcp/10501']
        }, {
          'peer_id': '2boNoMupBUEDxo8a1ZfXbDzVEhbDzBmwjSbNUjGM9BovL1vTeDS',
          'libp2p': '12D3KooWNZUqbSjuP5sbj2mxJ7uF2n7V9qVFMmMvoBLvVjQ4PuLZ',
          'key': '6a63e3fcbcc4b157fa979caee6a6b082915f9b4a338164419bf4b1736549911f',
          'dist': '7c72e046e581c92f68ec130f6d12a69a61994eaf3859f11f4d18778eab567f42',
          'addrs': ['/ip4/127.0.0.1/tcp/10514', '/ip4/34.27.206.250/tcp/10514', '/ip4/34.27.206.250/tcp/19751', '/ip4/10.4.69.10/tcp/10514']
        }, {
          'peer_id': '2b4u7TLFB2bx4k746JsQ58yMajxdHbsRS4rG3SoZgNJj7XgaTwF',
          'libp2p': '12D3KooWG4yo93FtFb9CAQPY6wZMTxJaxeJnE5psX67GQ8Q3u3im',
          'key': '6d759fadb2ee3044ffbfa199dbdc08bf76949f816586c681098521f37f3ce0b1',
          'dist': '7b649c17ebab483c6dc42e3850681ea786524a646e5e53dfdf69e70eb1230eec',
          'addrs': ['/ip4/127.0.0.1/tcp/10508', '/ip4/34.27.206.250/tcp/10508', '/ip4/34.27.206.250/tcp/50601', '/ip4/10.4.69.8/tcp/10508']
        }, {
          'peer_id': '2atkbNb1SpSe9pava34w47MzSFHtdhNWbNyBBZsuzbgFLv4uzVt',
          'libp2p': '12D3KooWEX3Rif4AQSQEVqThb16eGT1nd7Ggz31gzjjGTkpumGCG',
          'key': '736d3a011d21032dc24af9bdab796ea5324b9fa4dee61f4689524c43655b69ed',
          'dist': '657c39bb44647b555031761c20cd78bdc28d4a41d53e8a185fbe8abeab4487b0',
          'addrs': ['/ip4/127.0.0.1/tcp/10516', '/ip4/34.122.106.164/tcp/10516', '/ip4/34.122.106.164/tcp/21286', '/ip4/10.4.52.4/tcp/10516']
        }, {
          'peer_id': '2b1PhKMrhWtxheY9zEhQ34M2BempJqCN48AErM3dHvwQzfXCQGj',
          'libp2p': '12D3KooWFXtofKtdbG19NZK5T8EKKJjqu9HRKitcnDNMpQ5koMen',
          'key': '3b66757080259bc2f775ccf202fea416ea94d9763b5cd1c6c5de2015fac2e5d8',
          'dist': '2d7776cad960e3ba650e4353894ab20e1a520c93308444981332e6e834dd0b85',
          'addrs': ['/ip4/127.0.0.1/tcp/10502', '/ip4/34.171.103.38/tcp/10502', '/ip4/34.171.103.38/tcp/33561', '/ip4/10.4.146.14/tcp/10502']
        }]
      }]
    }).pipe(
      map((response: DhtPeersResponse) => this.mapDhtPeers(response))
    );
  }

  private mapDhtPeers(response: DhtPeersResponse): { peers: NetworkNodeDHT[], thisKey: string } {
    return {
      peers: response.buckets.reduce((acc, bucket) => {
        const nodes = bucket.entries.map(entry => {
          const binaryDistance = this.hexToBinary(entry.dist);
          return {
            peerId: entry.peer_id,
            addressesLength: entry.addrs.length,
            addrs: entry.addrs,
            key: entry.key,
            hexDistance: entry.dist,
            binaryDistance,
            xorDistance: entry.key === response.this_key ? '-' : this.getNumberOfZerosUntilFirst1(binaryDistance),
            bucketIndex: response.buckets.indexOf(bucket),
            bucketMaxHex: bucket.max_dist
          } as NetworkNodeDHT;
        });
        return acc.concat(nodes);
      }, []),
      thisKey: response.this_key
    };
  }

  private hexToBinary(hex: string): string {
    const decimalValue = BigInt('0x' + hex);
    const binaryString = decimalValue.toString(2);
    return binaryString.padStart(256, '0');
  }

  private getNumberOfZerosUntilFirst1(binaryString: string): number {
    let leadingZeros = 0;
    for (let i = 0; i < binaryString.length; i++) {
      if (binaryString[i] === '0') {
        leadingZeros++;
      } else {
        break;
      }
    }
    return leadingZeros;
  }
}

export interface DhtPeersResponse {
  this_key: string;
  buckets: Bucket[];
}

export interface Bucket {
  max_dist: string;
  entries: Entry[];
}

export interface Entry {
  peer_id: string;
  libp2p: string;
  key: string;
  dist: string;
  addrs: string[];
}
