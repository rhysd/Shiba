import * as github from 'hast-util-sanitize/lib/github';

/* tslint:disable:no-string-literal */
github.attributes['*'].push('className');
github.attributes['a'].push('alia-hidden');
/* tslint:enable:no-string-literal */

github.tagNames.push('input');
github.tagNames.push('span');
export default github;
