import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { catchError, Observable, of, tap } from 'rxjs';
import { SentryService } from '@core/services/sentry.service';

@Injectable({
  providedIn: 'root',
})
export class FirestoreService {
  private cloudFunctionUrl = 'https://us-central1-webnode-gtm-test.cloudfunctions.net/handleValidationAndStore';

  constructor(private sentryService: SentryService,
              private http: HttpClient) { }

  addHeartbeat(data: any): Observable<any> {
    console.log('Posting to cloud function:', data);
    return this.http.post(this.cloudFunctionUrl, { data })
      .pipe(
        tap(() => {
          this.sentryService.updateHeartbeat(data, data.submitter);
        }),
        catchError(error => {
          console.error('Error while posting heartbeat', error);
          return of(null);
        }),
      );
  }
}
