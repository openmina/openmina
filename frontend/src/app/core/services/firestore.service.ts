import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { catchError, Observable, of } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class FirestoreService {
  private cloudFunctionUrl =
    'https://us-central1-webnode-gtm-test.cloudfunctions.net/handleValidationAndStore';

  constructor(private http: HttpClient) {}

  addHeartbeat(data: any): Observable<any> {
    console.log('Posting to cloud function:', data);
    return this.http.post(this.cloudFunctionUrl, { data }).pipe(
      catchError(error => {
        console.error('Error while posting heartbeat', error);
        return of(null);
      }),
    );
  }
}
