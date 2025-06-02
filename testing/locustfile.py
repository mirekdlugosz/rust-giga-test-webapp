# locust configuration file
# Run it something like that:
#     locust -f testing/locustfile.py -H 'http://localhost:8088' -u 500 -r 40.0 -t 15m --headless --only-summary --html /tmp/locust-report.html NormalUser
#     locust -f testing/locustfile.py -H 'http://localhost:8088' -u 500 -r 40.0 -t 5m --headless --only-summary --html /tmp/locust-report.html AnsweringUser

import random
import time

from locust import HttpUser, task, between

question_ids = [
    'q1_1_0', 'q1_1_1', 'q1_1_2', 'q1_1_3', 'q1_1_4', 'q1_1_5', 'q1_1_6',
    'q1_1_7', 'q1_1_8', 'q1_1_9', 'q1_1_10', 'q1_1_11', 'q1_1_12',
    'q1_2_0', 'q1_2_1', 'q1_2_2', 'q1_2_3', 'q1_2_4', 'q1_2_5', 'q1_2_6',
    'q1_2_7', 'q1_2_8', 'q1_2_9', 'q1_2_10', 'q1_3_0', 'q1_3_1', 'q1_3_2',
    'q1_3_3', 'q1_3_4', 'q1_3_5', 'q1_4_0', 'q1_4_1', 'q1_4_2', 'q1_4_3',
    'q1_4_4', 'q1_4_5', 'q1_5_0', 'q1_5_1', 'q1_5_2', 'q1_5_3', 'q1_5_4',
    'q1_5_5', 'q1_6_0', 'q1_6_1', 'q1_6_2', 'q1_6_3', 'q1_6_4', 'q1_7_0',
    'q1_7_1', 'q1_8_0', 'q1_8_1', 'q1_8_2', 'q1_9_0', 'q1_9_1', 'q1_9_2',
    'q1_9_3', 'q1_9_4', 'q2_1_0', 'q2_1_1', 'q2_1_2', 'q2_1_3', 'q2_1_4',
    'q2_1_5', 'q2_1_6', 'q2_1_7', 'q2_2_0', 'q2_2_1', 'q2_2_2', 'q2_2_3',
    'q2_2_4', 'q2_2_5', 'q2_2_6', 'q2_3_0', 'q2_3_1', 'q2_3_2', 'q2_3_3',
    'q2_3_4', 'q2_3_5', 'q2_3_6', 'q2_3_7', 'q2_4_0', 'q2_4_1', 'q2_4_2',
    'q2_4_3', 'q2_4_4', 'q2_4_5', 'q2_4_6', 'q2_4_7', 'q2_4_8', 'q2_4_9',
    'q2_4_10', 'q2_4_11', 'q2_5_0', 'q2_5_1', 'q2_5_2', 'q2_5_3',
    'q2_5_4', 'q2_6_0', 'q2_6_1', 'q2_6_2', 'q2_6_3', 'q2_6_4', 'q2_6_5',
    'q2_6_6', 'q2_6_7', 'q2_7_0', 'q2_7_1', 'q2_7_2', 'q2_7_3', 'q2_7_4',
    'q2_7_5', 'q2_7_6', 'q2_7_7', 'q2_7_8', 'q2_8_0', 'q2_8_1', 'q2_8_2',
    'q2_9_0', 'q2_9_1', 'q2_9_2', 'q2_9_3', 'q2_9_4', 'q2_9_5', 'q2_9_6',
    'q3_1_0', 'q3_1_1', 'q3_1_2', 'q3_1_3', 'q3_1_4', 'q3_1_5', 'q3_1_6',
    'q3_1_7', 'q3_1_8', 'q3_1_9', 'q3_1_10', 'q3_1_11', 'q3_1_12',
    'q3_1_13', 'q3_1_14', 'q3_1_15', 'q3_1_16', 'q3_1_17', 'q3_1_18',
    'q3_1_19', 'q3_1_20', 'q3_1_21', 'q3_1_22', 'q3_1_23', 'q3_1_24',
    'q3_1_25', 'q3_1_26', 'q3_1_27', 'q3_1_28', 'q3_1_29', 'q3_1_30',
    'q3_1_31', 'q3_1_32', 'q4_1_0', 'q4_1_1', 'q4_1_2', 'q4_1_3',
    'q4_1_4', 'q4_1_5', 'q4_2_0', 'q4_2_1', 'q4_2_2', 'q4_2_3', 'q4_2_4',
    'q4_2_5', 'q4_2_6', 'q4_2_7', 'q4_2_8', 'q4_2_9', 'q4_2_10', 'q4_3_0',
    'q4_3_1', 'q4_3_2', 'q4_3_3', 'q4_3_4', 'q4_3_5', 'q4_3_6', 'q4_4_0',
    'q4_4_1', 'q4_4_2', 'q4_4_3', 'q4_4_4', 'q4_5_0', 'q4_5_1', 'q4_5_2',
    'q4_6_0', 'q4_6_1', 'q4_6_2', 'q4_6_3', 'q4_6_4', 'q4_6_5', 'q4_6_6',
    'q4_6_7', 'q4_6_8', 'q4_6_9', 'q4_6_10', 'q4_6_11', 'q4_7_0',
    'q4_7_1', 'q4_7_2', 'q4_7_3', 'q4_7_4', 'q4_7_5', 'q4_7_6', 'q4_8_0',
    'q4_8_1', 'q4_9_0', 'q4_9_1', 'q4_9_2', 'q4_9_3', 'q4_9_4', 'q4_9_5',
    'q4_9_6', 'q4_10_0', 'q4_10_1', 'q4_10_2', 'q4_10_3', 'q4_10_4',
    'q5_1_0', 'q5_1_1', 'q5_1_2', 'q5_1_3', 'q5_1_4', 'q5_1_5', 'q5_1_6',
    'q5_1_7', 'q5_1_8', 'q5_2_0', 'q5_2_1', 'q5_2_2', 'q5_2_3', 'q5_2_4',
    'q5_2_5', 'q5_3_0', 'q5_3_1', 'q5_3_2', 'q5_4_0', 'q5_4_1', 'q5_4_2',
    'q5_4_3', 'q5_4_4', 'q5_4_5', 'q5_5_0', 'q5_5_1', 'q5_5_2', 'q5_5_3',
    'q5_5_4', 'q5_6_0', 'q5_6_1', 'q6_1_0', 'q6_1_1', 'q6_1_2', 'q6_1_3',
    'q6_1_4', 'q6_1_5', 'q6_1_6', 'q6_1_7', 'q6_2_0', 'q6_2_1', 'q6_2_2',
    'q6_2_3', 'q6_2_4', 'q6_2_5', 'q6_2_6', 'q6_2_7', 'q6_2_8', 'q6_2_9',
    'q6_2_10', 'q6_2_11', 'q6_3_0', 'q6_3_1', 'q6_3_2', 'q6_3_3',
    'q6_3_4', 'q6_4_0', 'q6_4_1', 'q6_4_2', 'q6_5_0', 'q6_5_1', 'q6_5_2',
    'q6_5_3', 'q6_5_4', 'q6_5_5', 'q6_5_6', 'q6_5_7', 'q6_5_8', 'q6_5_9',
    'q6_6_0', 'q6_6_1', 'q6_6_2', 'q6_6_3', 'q6_6_4'
]
answers = ["A", "B", "C", "D"]
form_headers = {"Content-Type": "application/x-www-form-urlencoded"}


class NormalUser(HttpUser):
    def sleep(self):
        time.sleep(random.randint(1, 30))

    @task
    def user_journey(self):
        self.client.get("/")
        self.sleep()
        for part in range(1, random.randint(1, 7)):
            self.client.get(f"/czesc-{part}")
            self.sleep()
            max_ = int(part / 6 * len(question_ids))
            questions = random.sample(question_ids, k=random.randint(0, max_))
            payload = {
                qid: random.choice(answers) for qid in questions
            }
            self.client.post("/odpowiedzi", data=payload, headers=form_headers)
            self.client.get("/")
        self.sleep()
        self.client.post("/zakoncz", data={})
        self.client.get("/")


class AnsweringUser(HttpUser):
    @task
    def post_answer(self):
        questions = random.sample(question_ids, k=random.randint(0, len(question_ids)))
        payload = {
            qid: random.choice(answers) for qid in questions
        }
        self.client.post("/odpowiedzi", data=payload, headers=form_headers)
