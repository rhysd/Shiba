import * as React from 'react';
import * as fs from 'fs';
import * as unified from 'unified';
import * as parse from 'remark-parse';
import * as react from 'remark-react';
import * as toc from 'remark-toc';
import * as slug from 'remark-slug';
import * as headings from 'remark-autolink-headings';
import * as github from 'remark-github';
import * as lint from 'remark-lint';

const processor = unified({
    presets: ['lint-recommended'],
}).use([
    parse,
    react,
]).use(
    lint, {firstHeadingLevel: true}
).use([
    toc,
    slug,
    headings,
    github,
]);

interface Props extends React.Props<MarkdownTest> {
}

interface State {
    preview: React.ReactElement<any> | null;
}

export default class MarkdownTest extends React.Component<Props, State> {
    refs: {
        input: HTMLInputElement;
    };

    constructor(props: Props) {
        super(props);
        this.state = {
            preview: null,
        };
    }

    showPreview(file: string) {
        fs.readFile(file, 'utf8', (err, doc) => {
            if (err) {
                console.error(err);
                alert(err.message);
                return;
            }
            processor.process(doc, (e, result) => {
                if (e) {
                    console.error(e);
                    alert(e.message);
                    return;
                }
                console.log('Result:', result);
                this.setState({
                    preview: result.contents,
                });
            });
        });
    }

    componentDidMount() {
        this.refs.input.addEventListener('keydown', (e: KeyboardEvent) => {
            if (e.key === 'Enter') {
                const i = e.target as HTMLInputElement;
                this.showPreview(i.value);
            }
        });
        this.showPreview('../README.md');
    }

    render() {
        return <div>
            <input ref="input" />
            <div className="preview">
                {this.state.preview}
            </div>
        </div>;
    }
}
