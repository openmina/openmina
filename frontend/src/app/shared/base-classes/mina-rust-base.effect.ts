import { FeatureAction, MinaBaseEffect } from '@openmina/shared';
import { MinaState } from '@app/app.setup';

export abstract class MinaRustBaseEffect<A extends FeatureAction<any>> extends MinaBaseEffect<A, MinaState> {
}
